// Copyright 2020 Ministerie van Defensie
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::errors::ErrorKind::*;
use crate::errors::*;
use statsd::client::Pipeline;
use statsd::Client;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

pub mod errors;

///Delay used in the run loop of the statistics handler thread.
const STATS_DELAY_SEC: u64 = 1;

#[derive(Default)]
pub struct Counter(AtomicU64);

impl Counter {
    pub fn add(&self, value: u64) {
        self.0.fetch_add(value, Ordering::Relaxed);
    }
    pub fn load(&self) -> u64 {
        self.0.load(Ordering::Relaxed)
    }
    fn get_and_reset(&self) -> f64 {
        self.0.swap(0, Ordering::Relaxed) as f64
    }
}

#[derive(Default)]
pub struct Gauge(AtomicU64);

impl Gauge {
    pub fn set(&self, value: u64) {
        self.0.swap(value, Ordering::Relaxed);
    }
    fn get(&self) -> f64 {
        self.0.load(Ordering::Relaxed) as f64
    }
}

#[derive(Default)]
pub struct StatsAllHandlers {
    pub in_bytes: Counter,
    pub in_packets: Counter,
    pub out_bytes: Counter,
    pub out_packets: Counter,
    pub dropped_bytes: Counter,
    pub dropped_packets: Counter,
    pub packetloss: Counter,
    pub custom_counter: Option<(Counter, String)>,
    pub custom_gauge: Option<(Gauge, String)>,
}

impl StatisticData for StatsAllHandlers {
    fn fill_pipeline(&self, pipeline: &mut Pipeline) {
        pipeline.count(&"in.bytes", self.in_bytes.get_and_reset());
        pipeline.count(&"in.packets", self.in_packets.get_and_reset());
        pipeline.count(&"out.bytes", self.out_bytes.get_and_reset());
        pipeline.count(&"out.packets", self.out_packets.get_and_reset());
        pipeline.count(&"dropped.bytes", self.dropped_bytes.get_and_reset());
        pipeline.count(&"dropped.packets", self.dropped_packets.get_and_reset());
        pipeline.count(&"packetloss", self.packetloss.get_and_reset());
        if let Some(x) = &self.custom_counter {
            pipeline.count(&x.1, x.0.get_and_reset());
        }
        if let Some(x) = &self.custom_gauge {
            pipeline.gauge(&x.1, x.0.get());
        }
    }
    fn set_custom_gauge(&self, number: u64) -> Result<()> {
        match self.custom_gauge.as_ref() {
            Some(v) => v.0.set(number),
            None => return Err(CustomField("Custom field not set correct".to_string()).into()),
        }
        Ok(())
    }

    fn add_custom_counter(&self, number: u64) -> Result<()> {
        match self.custom_counter.as_ref() {
            Some(v) => v.0.add(number),
            None => return Err(CustomField("Custom field not set correct".to_string()).into()),
        }
        Ok(())
    }
}

///This trait is used to signal that a data struct can be passed to the StatsdClient
pub trait StatisticData {
    fn fill_pipeline(&self, pipeline: &mut Pipeline);
    fn set_custom_gauge(&self, number: u64) -> Result<()>;
    fn add_custom_counter(&self, number: u64) -> Result<()>;
}

///The statsdClient is used to send statistics data to the specified statsd server
pub struct StatsdClient<T: StatisticData + Send + Sync + 'static> {
    pub data: Arc<T>,
    is_running: Arc<AtomicBool>,
}

impl<T> StatsdClient<T>
where
    T: StatisticData + Send + Sync,
{
    ///Creates a new instance of the statsdClient
    pub fn new_standard() -> StatsdClient<StatsAllHandlers> {
        StatsdClient {
            data: Arc::new(StatsAllHandlers::default()),
            is_running: Arc::new(AtomicBool::default()),
        }
    }

    ///returns a clone of the data struct
    pub fn get_data_clone(&self) -> Arc<T> {
        self.data.clone()
    }

    pub fn new_with_custom_fields(
        custom_counter: Option<&str>,
        custom_gauge: Option<&str>,
    ) -> StatsdClient<StatsAllHandlers> {
        let mut counter_option: Option<(Counter, String)> = None;
        if let Some(x) = custom_counter {
            counter_option = Some((Counter::default(), x.to_string()));
        }
        let mut gauge_option: Option<(Gauge, String)> = None;
        if let Some(x) = custom_gauge {
            gauge_option = Some((Gauge::default(), x.to_string()));
        }
        StatsdClient {
            data: Arc::new(StatsAllHandlers {
                in_bytes: Counter::default(),
                in_packets: Counter::default(),
                out_bytes: Counter::default(),
                out_packets: Counter::default(),
                dropped_bytes: Counter::default(),
                dropped_packets: Counter::default(),
                custom_counter: counter_option,
                custom_gauge: gauge_option,
                packetloss: Counter::default(),
            }),
            is_running: Arc::new(AtomicBool::default()),
        }
    }

    ///This function is used to start the statsdClient.
    ///When this function is called the statsdClient starts sending statistics to the specified statsd server.
    /// # Arguments
    /// * `addr` - The address of the statsd server the statistics should be sent to.
    /// * `prefix` - The prefix that is used by statsd to name variables.
    /// # Returns
    /// * `JoinHandle` - The joinhandle of the thread that is created to run this function.
    pub fn run(&self, addr: String, prefix: String) -> std::io::Result<JoinHandle<()>> {
        let is_running = Arc::clone(&self.is_running);
        let data = Arc::clone(&self.data);
        thread::Builder::new()
            .name("statistics_handler_thread".into())
            .spawn(move || {
                statistics_inner_thread(is_running, addr, prefix, data)
                    .expect("Error in statitics thread");
            })
    }
    ///Stops the run loop of the statsdClient.
    pub fn stop(&self) {
        log::info!("Statistics Handler stopping.");
        self.is_running.store(false, Ordering::SeqCst);
    }
}

pub fn statistics_inner_thread(
    is_running: Arc<AtomicBool>,
    addr: String,
    prefix: String,
    data: Arc<dyn StatisticData + Send + Sync>,
) -> Result<()> {
    is_running.store(true, Ordering::SeqCst);
    match Client::new(addr, &prefix) {
        Ok(client) => {
            while is_running.load(Ordering::SeqCst) {
                let mut pipeline = client.pipeline();
                data.fill_pipeline(&mut pipeline);
                pipeline.send(&client);
                std::thread::sleep(std::time::Duration::from_secs(STATS_DELAY_SEC));
            }
        }
        Err(e) => return Err(Error::with_chain(e, "Failed to create StatsD Client")),
    };
    Ok(())
}
