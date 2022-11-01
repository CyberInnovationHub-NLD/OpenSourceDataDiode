-------------------------------------------------------------------------------
-- Title      : diode
-------------------------------------------------------------------------------
-- Description: Control startup (or reset)
-- summary startup sequence.

--  reset_after 10clocks op (25mHZ == 40ns * 10 = 400ns) => (125mHZ == 8ns   400ns / 8ns = 50 (0b110010) clock cycles op de system clock)

-- 0  0b 00000 (0x00) w 0x1140 \n  -- phy config ok                                       -0
-- 16 0b 10000 (0x40) w 0x60   \n  -- For MDI Crossover mode setting                      -1
-- 20 0b 10100 (0x50) w 0xce2  \n  -- to add delay option                                 -2
-- 18 0b 10010        w 0x4400 \n  -- interupt enables !                                  -3 
-- 27 0b 11011 (0x6c) w 0x840b \n  -- RGMII to copper (840b - for interupt polatrity)     -4
-- 0  0b 00000 (0x00) w 0x9140 \n  -- software reset                                      -5
-- 4  0b 00100        w 0xde1  \n  -- set pause frame

--not used 
--  8  0b 00100 (0x10) w 0x1    \n  -- for GB only (delete if all speed !)
-------------------------------------------------------------------------------
-------------------------------------------------------------------------------
library ieee;
use ieee.std_logic_1164.all;
use ieee.std_logic_unsigned.all;
use ieee.numeric_std.all;

library work;

entity OSDD_control is
      generic (
            g_simulation     : boolean                      := false;
            g_phy_0_speed    : std_logic_vector(1 downto 0) := "10";
            g_phy_1_speed    : std_logic_vector(1 downto 0) := "10";
            g_phya_addr      : std_logic_vector             := "00000";
            g_phyb_addr      : std_logic_vector             := "00001";
            g_shared_mdiobus : boolean                      := true --if false both phys have address a
      );
      port (
            clk   : in std_logic; -- system clock 125mHZ
            reset : in std_logic; -- raw reset

            mdio_phy_addres : out std_logic_vector(4 downto 0);
            mdio_address    : out std_logic_vector(4 downto 0);
            mdio_data_tx    : out std_logic_vector(15 downto 0);
            mdio_data_rx    : in std_logic_vector(15 downto 0);
            mdio_read       : out std_logic;
            mdio_start      : out std_logic;
            mdio_ready      : in std_logic;

            reset_phy        : out std_logic;
            speed_stat_phy_0 : out std_logic_vector(1 downto 0);
            speed_stat_phy_1 : out std_logic_vector(1 downto 0);
            interrupt_phy_0  : in std_logic;
            interrupt_phy_1  : in std_logic --can be disconnected connect '0'
      );
end entity;

architecture behavior of OSDD_control is

      ------ Counter for flow control ------
      signal counter     : std_logic_vector(31 downto 0) := (others => '0'); --if full then trow reset_phy
      signal counter_en  : std_logic                     := '0';
      signal counter_rst : std_logic                     := '0';
      signal send_number : std_logic_vector(3 downto 0)  := (others => '0');

      signal speed_stat_phy_0_reg : std_logic_vector(1 downto 0) := "11";
      signal speed_stat_phy_1_reg : std_logic_vector(1 downto 0) := "11";
      signal speed_stat_phy_0_old : std_logic_vector(1 downto 0) := "11";
      signal speed_stat_phy_1_old : std_logic_vector(1 downto 0) := "11";

      signal reading_phy, reading_phy_old : std_logic := '0';
      ------ State machine ------
      type State_type is (idle, phy_rst, startup, W_mdio, read_17, W_mdio_read_17, read_19, W_mdio_read_19, done); -- Define the states
      signal State, next_State : State_Type;                                                                       -- Create a signal that uses

begin
      speed_stat_phy_0 <= speed_stat_phy_0_old;
      speed_stat_phy_1 <= speed_stat_phy_1_old;
      process (clk, reset)
      begin
            if (reset = '1') then
                  State                <= idle;
                  counter              <= (others => '0');
                  send_number          <= "0000";
                  speed_stat_phy_0_old <= "11";
                  speed_stat_phy_1_old <= "11";
                  reading_phy_old      <= '0';
            elsif (rising_edge(clk)) then
                  if (counter_rst = '1') then
                        counter <= (others => '0');
                  elsif (counter_en = '1') then
                        counter <= counter + "1";
                  end if;
                  if (mdio_ready = '1') then
                        if (g_shared_mdiobus) then
                              send_number <= send_number + "1";
                        else
                              send_number <= send_number + "10";
                        end if;
                  end if;
                  speed_stat_phy_0_old <= speed_stat_phy_0_reg;
                  speed_stat_phy_1_old <= speed_stat_phy_1_reg;
                  reading_phy_old      <= reading_phy;
                  State                <= next_State;
            end if;
      end process;

      ------ STATE MACHINE ------
      process (State, counter, send_number, mdio_ready, speed_stat_phy_0_old, speed_stat_phy_1_old, interrupt_phy_0, interrupt_phy_1, reading_phy_old, mdio_data_rx)
      begin
            case State is
                  when idle =>
                        counter_en  <= '1';
                        counter_rst <= '0';
                        reset_phy   <= '1';

                        speed_stat_phy_0_reg <= "00";
                        speed_stat_phy_1_reg <= "00";
                        reading_phy          <= '0';

                        mdio_phy_addres <= "00000";
                        mdio_address    <= "00000";
                        mdio_data_tx    <= x"0000";
                        mdio_read       <= '0';
                        mdio_start      <= '0';

                        if (counter >= "111111111111") then
                              next_State <= phy_rst;
                        else
                              next_State <= idle;
                        end if;

                  when phy_rst =>
                        counter_en  <= '1';
                        counter_rst <= '0';
                        reset_phy   <= '0';

                        speed_stat_phy_0_reg <= speed_stat_phy_0_old;
                        speed_stat_phy_1_reg <= speed_stat_phy_1_old;
                        reading_phy          <= reading_phy_old;

                        mdio_phy_addres <= "00000";
                        mdio_address    <= "00000";
                        mdio_data_tx    <= x"0000";
                        mdio_read       <= '0';
                        mdio_start      <= '0';

                        if (counter >= "1111111111111111") then
                              next_State <= startup;
                        else
                              next_State <= phy_rst;
                        end if;

                  when startup =>
                        counter_en  <= '0';
                        counter_rst <= '1';
                        reset_phy   <= '0';

                        speed_stat_phy_0_reg <= speed_stat_phy_0_old;
                        speed_stat_phy_1_reg <= speed_stat_phy_1_old;
                        reading_phy          <= reading_phy_old;

                        case send_number is
                              when "0000" => --0
                                    mdio_phy_addres <= g_phya_addr;
                                    mdio_address    <= "00000";
                                    mdio_data_tx    <= x"1140";
                                    mdio_read       <= '0';
                                    mdio_start      <= '1';
                                    next_State      <= W_mdio;
                              when "0001" =>
                                    mdio_phy_addres <= g_phyb_addr;
                                    mdio_address    <= "00000";
                                    mdio_data_tx    <= x"1140";
                                    mdio_read       <= '0';
                                    mdio_start      <= '1';
                                    next_State      <= W_mdio;

                              when "0010" => -- 1
                                    mdio_phy_addres <= g_phya_addr;
                                    mdio_address    <= "10000";
                                    mdio_data_tx    <= x"0060";
                                    mdio_read       <= '0';
                                    mdio_start      <= '1';
                                    next_State      <= W_mdio;
                              when "0011" =>
                                    mdio_phy_addres <= g_phyb_addr;
                                    mdio_address    <= "10000";
                                    mdio_data_tx    <= x"0060";
                                    mdio_read       <= '0';
                                    mdio_start      <= '1';
                                    next_State      <= W_mdio;

                              when "0100" => --2
                                    mdio_phy_addres <= g_phya_addr;
                                    mdio_address    <= "10100";
                                    mdio_data_tx    <= x"0ce2";
                                    mdio_read       <= '0';
                                    mdio_start      <= '1';
                                    next_State      <= W_mdio;
                              when "0101" =>
                                    mdio_phy_addres <= g_phyb_addr;
                                    mdio_address    <= "10100";
                                    mdio_data_tx    <= x"0ce2";
                                    mdio_read       <= '0';
                                    mdio_start      <= '1';
                                    next_State      <= W_mdio;

                                    -- 48 0b 10010        w 0x4400 \n  -- interupt enables !
                                    -- 8  0b 00100 (0x10) w 0x1    \n  -- for GB only (delete if all speed ~! checken of half duplex werkt.)
                              when "0110" => --3
                                    mdio_phy_addres <= g_phya_addr;
                                    mdio_address    <= "10010";
                                    mdio_data_tx    <= x"4400";
                                    mdio_read       <= '0';
                                    mdio_start      <= '1';
                                    next_State      <= W_mdio;
                              when "0111" =>
                                    mdio_phy_addres <= g_phyb_addr;
                                    mdio_address    <= "10010";
                                    mdio_data_tx    <= x"4400";
                                    mdio_read       <= '0';
                                    mdio_start      <= '1';
                                    next_State      <= W_mdio;

                              when "1000" => --4
                                    mdio_phy_addres <= g_phya_addr;
                                    mdio_address    <= "11011";
                                    mdio_data_tx    <= x"840b"; --intenable (840b)
                                    mdio_read       <= '0';
                                    mdio_start      <= '1';
                                    next_State      <= W_mdio;
                              when "1001" =>
                                    mdio_phy_addres <= g_phyb_addr;
                                    mdio_address    <= "11011";
                                    mdio_data_tx    <= x"840b";
                                    mdio_read       <= '0';
                                    mdio_start      <= '1';
                                    next_State      <= W_mdio;

                              when "1010" => --5
                                    mdio_phy_addres <= g_phya_addr;
                                    mdio_address    <= "00000";
                                    mdio_data_tx    <= x"9140";
                                    mdio_read       <= '0';
                                    mdio_start      <= '1';
                                    next_State      <= W_mdio;
                              when "1011" =>
                                    mdio_phy_addres <= g_phyb_addr;
                                    mdio_address    <= "00000";
                                    mdio_data_tx    <= x"9140";
                                    mdio_read       <= '0';
                                    mdio_start      <= '1';
                                    next_State      <= W_mdio;

                              when others =>
                                    mdio_phy_addres <= "00000";
                                    mdio_address    <= "00000";
                                    mdio_data_tx    <= x"0000";
                                    mdio_read       <= '0';
                                    mdio_start      <= '0';
                                    next_State      <= done;
                        end case;

                  when W_mdio =>
                        counter_en  <= '0';
                        counter_rst <= '1';
                        reset_phy   <= '0';

                        speed_stat_phy_0_reg <= speed_stat_phy_0_old;
                        speed_stat_phy_1_reg <= speed_stat_phy_1_old;
                        reading_phy          <= reading_phy_old;

                        mdio_phy_addres <= "00000";
                        mdio_address    <= "00000";
                        mdio_data_tx    <= x"0000";
                        mdio_read       <= '0';
                        mdio_start      <= '0';

                        if (mdio_ready = '1') then
                              next_State <= startup;
                        else
                              next_State <= W_mdio;
                        end if;

                  when read_17 =>
                        counter_en  <= '0';
                        counter_rst <= '1';
                        reset_phy   <= '0';
                        if (reading_phy_old = '1') then
                              mdio_phy_addres <= g_phyb_addr;
                        else
                              mdio_phy_addres <= g_phya_addr;
                        end if;
                        speed_stat_phy_0_reg <= speed_stat_phy_0_old;
                        speed_stat_phy_1_reg <= speed_stat_phy_1_old;
                        reading_phy          <= reading_phy_old;
                        mdio_address         <= "10001";
                        mdio_data_tx         <= x"0000";
                        mdio_read            <= '1';
                        mdio_start           <= '1';
                        next_State           <= W_mdio_read_17;

                  when W_mdio_read_17 =>
                        counter_en  <= '0';
                        counter_rst <= '1';
                        reset_phy   <= '0';

                        mdio_phy_addres <= "00000";
                        mdio_address    <= "00000";
                        mdio_data_tx    <= x"0000";
                        mdio_read       <= '0';
                        mdio_start      <= '0';

                        if (mdio_ready = '1') then
                              if (reading_phy_old = '1') then
                                    speed_stat_phy_1_reg <= mdio_data_rx(15 downto 14);
                                    speed_stat_phy_0_reg <= speed_stat_phy_0_old;
                              else
                                    speed_stat_phy_0_reg <= mdio_data_rx(15 downto 14);
                                    speed_stat_phy_1_reg <= speed_stat_phy_1_old;
                              end if;
                              next_State <= read_19;
                        else
                              next_State           <= W_mdio_read_17;
                              speed_stat_phy_0_reg <= speed_stat_phy_0_old;
                              speed_stat_phy_1_reg <= speed_stat_phy_1_old;
                        end if;
                        reading_phy <= reading_phy_old;

                  when read_19 =>
                        counter_en  <= '0';
                        counter_rst <= '1';
                        reset_phy   <= '0';

                        if (reading_phy_old = '1') then
                              mdio_phy_addres <= g_phyb_addr;
                        else
                              mdio_phy_addres <= g_phya_addr;
                        end if;

                        speed_stat_phy_0_reg <= speed_stat_phy_0_old;
                        speed_stat_phy_1_reg <= speed_stat_phy_1_old;
                        reading_phy          <= reading_phy_old;

                        mdio_address <= "10011";
                        mdio_data_tx <= x"0000";
                        mdio_read    <= '1';
                        mdio_start   <= '1';

                        next_State <= W_mdio_read_19;

                  when W_mdio_read_19 =>
                        counter_en  <= '0';
                        counter_rst <= '1';
                        reset_phy   <= '0';

                        speed_stat_phy_0_reg <= speed_stat_phy_0_old;
                        speed_stat_phy_1_reg <= speed_stat_phy_1_old;
                        reading_phy          <= reading_phy_old;

                        mdio_phy_addres <= "00000";
                        mdio_address    <= "00000";
                        mdio_data_tx    <= x"0000";
                        mdio_read       <= '0';
                        mdio_start      <= '0';

                        if (mdio_ready = '1') then
                              next_State <= done;
                        else
                              next_State <= W_mdio_read_19;
                        end if;

                  when done =>
                        counter_en  <= '0';
                        counter_rst <= '1';
                        reset_phy   <= '0';
                        if (g_simulation = true) then
                              speed_stat_phy_0_reg <= g_phy_0_speed;
                              speed_stat_phy_1_reg <= g_phy_1_speed;
                        else
                              speed_stat_phy_0_reg <= speed_stat_phy_0_old;
                              speed_stat_phy_1_reg <= speed_stat_phy_1_old;
                        end if;

                        mdio_phy_addres <= "00000";
                        mdio_address    <= "00000";
                        mdio_data_tx    <= x"0000";
                        mdio_read       <= '0';
                        mdio_start      <= '0';
                        reading_phy     <= interrupt_phy_1;
                        if (interrupt_phy_0 = '1' or interrupt_phy_1 = '1') then
                              next_State <= read_17;
                        else
                              next_State <= done;
                        end if;
                  when others =>
                        next_State <= idle;

            end case;
      end process;
end behavior;