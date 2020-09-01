-------------------------------------------------------------------------------
-- Title      : OSDD_MDIO_interface.vhd
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
-- A MDIO transaction is 64 MDC clock cycles long and contains 32 cycles
-- preamble, 2 cycles start of frame, 2 cycles op code, 5 cycles port address,
-- 5 cycles device address, 2 cycles turnaround and 16 cycles of address/data.
-------------------------------------------------------------------------------
library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

entity OSDD_MDIO_interface is
      generic (
            g_clock_frequency : integer := 125000000; -- system clock frequency in Hz
            g_mdc_frequency   : integer := 2500000;   -- maximum mdc clock frequency in Hz
            g_rising_edge     : boolean := false;     -- mdc clock edge which data is changed (true = rising edge, false = falling edge)
            -- mdio specifics
            g_preamble      : std_logic_vector := "01";
            g_read_command  : std_logic_vector := "10";
            g_write_command : std_logic_vector := "01";
            g_turnaround    : std_logic_vector := "10"
      );
      port (
            clock : in std_logic; -- system clock
            reset : in std_logic; -- stystem reset
            -- MDIO_data interface
            mdio_phy_addres : in std_logic_vector(4 downto 0);
            mdio_address    : in std_logic_vector(4 downto 0);   -- stable with start
            mdio_data_tx    : in std_logic_vector(15 downto 0);  -- stable with start
            mdio_data_rx    : out std_logic_vector(15 downto 0); -- stable with ready
            mdio_read       : in std_logic;                      -- full cycle high
            mdio_start      : in std_logic;                      -- pulse
            mdio_ready      : out std_logic;                     -- pulse
            -- Eth_MDIO interfae
            mdc    : out std_logic;
            mdio_i : in std_logic;
            mdio_o : out std_logic;
            mdio_t : out std_logic
      );
end entity;

architecture behavior of OSDD_MDIO_interface is

      constant c_timeout  : integer := g_clock_frequency; -- Timeout after a mdio operation was started (1 sec.)
      constant c_division : integer := ((g_clock_frequency + g_mdc_frequency - 1) / g_mdc_frequency);

      type t_mdio_state is (idle, preamble, run, done);
      signal mdio_state : t_mdio_state;

      signal mdio_data_out : std_logic_vector(31 downto 0); -- outgoing data
      signal mdio_data_in  : std_logic_vector(15 downto 0); -- incomming data.
      signal mdio_out      : std_logic;                     -- intern sync

      signal clock_div : natural range 0 to (c_division - 1);
      signal length    : natural range 0 to 31;
      signal tri_state : boolean;
      signal next_bit  : std_logic;

begin
      ---------------------------------------------------------------------------
      -- Generate the MDIO clock and sample the input.
      ---------------------------------------------------------------------------
      gen_sample_at_rising_edge : if (g_rising_edge) generate
            process (clock)
            begin
                  if (rising_edge(clock)) then
                        next_bit <= '0';

                        if (clock_div = (c_division - 1)) then
                              mdc          <= '0';
                              clock_div    <= 0;
                              mdio_data_in <= mdio_data_in(mdio_data_in'high - 1 downto 0) & mdio_i;
                        else
                              clock_div <= clock_div + 1;
                              if (clock_div = ((c_division / 2) - 1)) then
                                    next_bit <= '1';
                                    mdc      <= '1';
                                    mdio_o   <= mdio_out;
                                    if tri_state then
                                          mdio_t <= '1';
                                    else
                                          mdio_t <= '0';
                                    end if;
                              end if;
                        end if;

                        if (reset = '1') then
                              next_bit     <= '0';
                              clock_div    <= 0;
                              mdio_data_in <= (others => '0');
                        end if;
                  end if;
            end process;
      end generate;

      gen_sample_at_falling_edge : if (not g_rising_edge) generate
            process (clock)
            begin
                  if (rising_edge(clock)) then
                        next_bit <= '0';

                        if (clock_div = (c_division - 1)) then
                              next_bit  <= '1';
                              mdc       <= '0';
                              clock_div <= 0;
                              mdio_o    <= mdio_out;
                              if tri_state then
                                    mdio_t <= '1';
                              else
                                    mdio_t <= '0';
                              end if;
                        else
                              clock_div <= clock_div + 1;
                              if (clock_div = ((c_division / 2) - 1)) then
                                    mdc          <= '1';
                                    mdio_data_in <= mdio_data_in(mdio_data_in'high - 1 downto 0) & mdio_i;
                              end if;
                        end if;

                        if (reset = '1') then
                              next_bit     <= '0';
                              clock_div    <= 0;
                              mdio_data_in <= (others => '0');
                        end if;
                  end if;
            end process;
      end generate;

      ---------------------------------------------------------------------------
      -- Process which handles the MDIO operations.
      ---------------------------------------------------------------------------
      process (clock)
      begin
            if (rising_edge(clock)) then
                  mdio_ready <= '0';
                  case mdio_state is
                        when idle =>
                              length <= 0;
                              if (mdio_start = '1') then
                                    mdio_out  <= '1';
                                    tri_state <= false;
                                    -- build output vector
                                    if (mdio_read = '1') then
                                          mdio_data_out <= g_preamble & g_read_command & mdio_phy_addres & mdio_address & g_turnaround & "0000000000000000";
                                    else
                                          mdio_data_out <= g_preamble & g_write_command & mdio_phy_addres & mdio_address & g_turnaround & mdio_data_tx;
                                    end if;
                                    mdio_state <= preamble;
                              end if;

                              -- 32 bits preamble
                        when preamble =>
                              if (next_bit = '1') then
                                    if (length /= 31) then
                                          length <= length + 1;
                                    else
                                          length     <= 0;
                                          mdio_state <= run;
                                    end if;
                              end if;

                              -- shift 16 or 32 bits out and 16 or 0 bits in (read respectively
                              -- write operation)
                        when run =>
                              mdio_out <= mdio_data_out(31 - length);
                              if (next_bit = '1') then
                                    if (length /= 31) then
                                          length <= length + 1;
                                    else
                                          tri_state  <= true;
                                          mdio_state <= done;
                                    end if;

                                    -- turnaround -> at the 15th clock pulse the line must be
                                    -- held tri-state (in case of a read operation also the
                                    -- last 16 clock pulses the line must be held in tri-state).
                                    case length is
                                          when 13 => tri_state <= (mdio_read = '1');

                                          when others => null;
                                    end case;
                              end if;

                        when done =>
                              if (next_bit = '1') then
                                    mdio_data_rx <= mdio_data_in;
                                    mdio_ready   <= '1';
                                    tri_state    <= true;
                                    mdio_state   <= idle;
                              end if;

                        when others =>
                              mdio_state <= idle;

                  end case;

                  if (reset = '1') then
                        mdio_data_out <= (others => '0');
                        mdio_ready    <= '0';
                        mdio_state    <= idle;
                        tri_state     <= true;
                        mdio_out      <= '1';
                        length        <= 0;
                        mdio_data_rx  <= (others => '0');
                  end if;
            end if;
      end process;

end architecture;