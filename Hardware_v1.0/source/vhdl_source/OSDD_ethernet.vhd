-------------------------------------------------------------------------------
-- Title      : OSDD_ethernet
-------------------------------------------------------------------------------
-- Description:
-------------------------------------------------------------------------------
library ieee;
use ieee.std_logic_1164.all;
use ieee.std_logic_unsigned.all;
use ieee.numeric_std.all;

library work;
entity OSDD_ethernet is
      generic (
            g_clock_frequency : integer          := 125000000; -- system clock
            g_mdc_frequency   : integer          := 2500000;   -- mdio clock
            g_phya_addr       : std_logic_vector := "00000";
            g_phyb_addr       : std_logic_vector := "00001";
            g_shared_mdiobus  : boolean          := true --if false both phys have address a
      );
      port (
            clk : in std_logic := '0';

            phya_rx_clk : in std_logic  := '0';
            phyb_tx_clk : out std_logic := '0';

            reset       : in std_logic  := '0'; -- raw reset.
            reset_phy_a : out std_logic := '0'; -- phy0 reset.
            reset_phy_b : out std_logic := '0'; -- phy1 reset.

            phya_rx_vec : in std_logic_vector(9 downto 0);
            phyb_tx_vec : out std_logic_vector(9 downto 0);

            interrupt_phy_0 : in std_logic := '0';
            interrupt_phy_1 : in std_logic := '0';

            mdio_i   : in std_logic  := '0';
            mdio_o   : out std_logic := '0';
            mdio_t   : out std_logic := '0';
            mdio_mdc : out std_logic := '0';

            speed_stat_phy_0_d : out std_logic_vector(1 downto 0);
            speed_stat_phy_1_d : out std_logic_vector(1 downto 0)
      );
end entity;

architecture structural of OSDD_ethernet is

      ------ Control signals -------
      signal speed_stat_phy_0, speed_stat_phy_1               : std_logic_vector(1 downto 0) := "11";
      signal speed_stat_phy_0_sync_1, speed_stat_phy_1_sync_1 : std_logic_vector(1 downto 0) := "11";
      signal speed_stat_phy_0_sync, speed_stat_phy_1_sync     : std_logic_vector(1 downto 0) := "11";
      signal data_out_valid                                   : std_logic                    := '0';

      ------ MDIO control signals --------
      signal mdio_phy_addres : std_logic_vector(4 downto 0)  := (others => '0');
      signal mdio_address    : std_logic_vector(4 downto 0)  := (others => '0');
      signal mdio_data_tx    : std_logic_vector(15 downto 0) := (others => '0');
      signal mdio_data_rx    : std_logic_vector(15 downto 0) := (others => '0');
      signal mdio_read       : std_logic                     := '0';
      signal mdio_start      : std_logic                     := '0';
      signal mdio_ready      : std_logic                     := '0';
      signal reset_phy       : std_logic                     := '0';

begin
      speed_stat_phy_0_d <= speed_stat_phy_0;
      speed_stat_phy_1_d <= speed_stat_phy_1;
      -----------------------------------------------------------
      -- connect rgmii interfaces                               --
      -----------------------------------------------------------
      process (phya_rx_clk)
      begin
            if (rising_edge(phya_rx_clk)) then

                  speed_stat_phy_0_sync_1 <= speed_stat_phy_0;
                  speed_stat_phy_0_sync   <= speed_stat_phy_0_sync_1;
                  speed_stat_phy_1_sync_1 <= speed_stat_phy_1;
                  speed_stat_phy_1_sync   <= speed_stat_phy_1_sync_1;

                  if (speed_stat_phy_0_sync = speed_stat_phy_1_sync and speed_stat_phy_0_sync /= "11") then
                        phyb_tx_vec <= phya_rx_vec;
                  else
                        phyb_tx_vec <= (others => '0');
                  end if;
            end if;
      end process;
      phyb_tx_clk <= phya_rx_clk;
      -----------------------------------------------------------
      -- Controller                                            --
      -----------------------------------------------------------
      reset_phy_a <= reset_phy;
      reset_phy_b <= reset_phy;
      control_phy0 : entity work.OSDD_control
            generic map(
                  g_phya_addr      => g_phya_addr,
                  g_phyb_addr      => g_phyb_addr,
                  g_shared_mdiobus => g_shared_mdiobus
            )
            port map(
                  clk   => clk,
                  reset => reset,

                  mdio_phy_addres => mdio_phy_addres,
                  mdio_address    => mdio_address,
                  mdio_data_tx    => mdio_data_tx,
                  mdio_data_rx    => mdio_data_rx,
                  mdio_read       => mdio_read,
                  mdio_start      => mdio_start,
                  mdio_ready      => mdio_ready,

                  reset_phy        => reset_phy,
                  speed_stat_phy_0 => speed_stat_phy_0,
                  speed_stat_phy_1 => speed_stat_phy_1,
                  interrupt_phy_0  => interrupt_phy_0,
                  interrupt_phy_1  => interrupt_phy_1
            );

      -----------------------------------------------------------
      -- MDIO INTERFACING                                      --
      -----------------------------------------------------------
      mdio_phy0 : entity work.OSDD_MDIO_interface
            port map(
                  clock => clk,
                  reset => reset,
                  -- MDIO_data interface
                  mdio_phy_addres => mdio_phy_addres,
                  mdio_address    => mdio_address,
                  mdio_data_tx    => mdio_data_tx,
                  mdio_data_rx    => mdio_data_rx,
                  mdio_read       => mdio_read,
                  mdio_start      => mdio_start,
                  mdio_ready      => mdio_ready,
                  -- Eth_MDIO interfae
                  mdc    => mdio_mdc,
                  mdio_i => mdio_i,
                  mdio_o => mdio_o,
                  mdio_t => mdio_t
            );
end architecture;