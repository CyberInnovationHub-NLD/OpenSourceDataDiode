library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;
use ieee.std_logic_unsigned.all;

entity OSDD_top_tb is
      generic (
            g_clock_frequency : integer                      := 125000000; -- system clock
            g_mdc_frequency   : integer                      := 2500000;   -- mdio clock
            g_simulation      : boolean                      := true;
            g_phy_0_speed     : std_logic_vector(1 downto 0) := "01";
            g_phy_1_speed     : std_logic_vector(1 downto 0) := "01"
      );
end;

signal CLK_LVDS_125_P : std_logic                    := '0';
signal FPGA_RESETN    : std_logic                    := '0';
signal USER_LED       : std_logic_vector(4 downto 0) := (others => '0')';

------ PHY_A ------
signal ENETA_INTN        : std_logic                    := '0';
signal ENETA_RESETN      : std_logic                    := '0';
signal ENETA_RX_CLK      : std_logic                    := '0';
signal ENETA_RX_COL      : std_logic                    := '0';
signal ENETA_RX_CRS      : std_logic                    := '0';
signal ENETA_RX_D        : std_logic_vector(3 downto 0) := (others => '0')';
signal ENETA_RX_DV       : std_logic                    := '0';
signal ENETA_RX_ER       : std_logic                    := '0';
signal ENETA_GTX_CLK     : std_logic                    := '0';
signal ENETA_TX_D        : std_logic_vector(3 downto 0) := (others => '0')';
signal ENETA_TX_EN       : std_logic                    := '0';
signal ENETA_TX_ER       : std_logic                    := '0';
signal ENETA_LED_LINK100 : std_logic                    := '0';

------ PHY_B ------
signal ENETB_INTN        : std_logic                    := '0';
signal ENETB_RESETN      : std_logic                    := '0';
signal ENETB_RX_CLK      : std_logic                    := '0';
signal ENETB_RX_COL      : std_logic                    := '0';
signal ENETB_RX_CRS      : std_logic                    := '0';
signal ENETB_RX_D        : std_logic_vector(3 downto 0) := (others => '0')';
signal ENETB_RX_DV       : std_logic                    := '0';
signal ENETB_RX_ER       : std_logic                    := '0';
signal ENETB_GTX_CLK     : std_logic                    := '0';
signal ENETB_TX_D        : std_logic_vector(3 downto 0) := (others => '0')';
signal ENETB_TX_EN       : std_logic                    := '0';
signal ENETB_TX_ER       : std_logic                    := '0';
signal ENETB_LED_LINK100 : std_logic                    := '0';

------ MDIO to PHY ------
signal ENET_MDC  : std_logic := '0';
signal ENET_MDIO : std_logic := '0';

------ Counter for flow control ------
signal packet_size_c    : std_logic_vector(7 downto 0) := (others => '0'); --if full then trow reset_phy
signal interframe_gap_c : std_logic_vector(7 downto 0) := (others => '0'); --12 CLOCK CYCLES! time it take to trasmit 96 bits
signal check_c          : std_logic_vector(7 downto 0) := (others => '0'); --12 CLOCK CYCLES! time it take to trasmit 96 bits
begin
CLK_LVDS_125_P <= not CLK_LVDS_125_P after 4 ns; --125MHZ
FPGA_RESETN    <= '1', '0' after 400 ns;
ENETA_RX_CLK   <= not ENETA_RX_CLK after 4 ns;

process (CLK_LVDS_125_P, ext_reset)
begin
      if (ext_reset = '1') then
            CLK_LVDS_125_P   <= "0000";
            packet_size_c    <= (others => '0');
            interframe_gap_c <= (others => '0');
      elsif (rising_edge(CLK_LVDS_125_P)) then
            if (packet_size_c >= "1111") then

            elsif (interframe_gap_c < "00001100") then
                  interframe_gap_c <= interframe_gap_c + "00000001";
                  ENETA_RX_D       <= (others => '0');
                  ENETA_RX_DV      <= '0';
            elsif (USER_LED(0) = "1" and NET0_RX_D >= packet_size_c) then
                  packet_size_c    <= packet_size_c + 1;
                  interframe_gap_c <= (others => '0');
                  ENETA_RX_DV      <= '0';
            elsif (USER_LED(0) = "1" and ENETA_RX_D < packet_size_c) then
                  ENETA_RX_DV <= '1';
                  ENETA_RX_D  <= ENETA_RX_D + "0001";
            end if;
      end if;
end process;

-- process(NET1_GTX_CLK,ext_reset)
-- begin
--   if (ext_reset = '1' or NET1_TX_EN = '0') then
--     check_c <= "00000001";
--   elsif (rising_edge(NET1_GTX_CLK)) then
--       if (NET1_TX_EN = '1') then
--           check_c <= check_c + "00000001";
--           if (NET1_TX_D /= check_c) then
--             report "output is not valid" severity error;
--           elsif (NET1_TX_D = "11111110") then   --0xFE
--             report "passed simulation" severity note;
--           end if;
--       end if;
--   end if;
-- end process;

OSDDV1 : entity work.OSDD_top
      port map(
            CLK_LVDS_125_P => CLK_LVDS_125_P,
            FPGA_RESETN    => FPGA_RESETN,
            USER_LED       => USER_LED,

            ------ PHY_A ------
            ENETA_INTN        => ENETA_INTN,
            ENETA_RESETN      => ENETA_RESETN,
            ENETA_RX_CLK      => ENETA_RX_CLK,
            ENETA_RX_COL      => ENETA_RX_COL,
            ENETA_RX_CRS      => ENETA_RX_CRS,
            ENETA_RX_D        => ENETA_RX_D,
            ENETA_RX_DV       => ENETA_RX_DV,
            ENETA_RX_ER       => ENETA_RX_ER,
            ENETA_GTX_CLK     => ENETA_GTX_CLK,
            ENETA_TX_D        => ENETA_TX_D,
            ENETA_TX_EN       => ENETA_TX_EN,
            ENETA_TX_ER       => ENETA_TX_ER,
            ENETA_LED_LINK100 => ENETA_LED_LINK100,

            ------ PHY_B ------
            ENETB_INTN        => ENETB_INTN,
            ENETB_RESETN      => ENETB_RESETN,
            ENETB_RX_CLK      => ENETB_RX_CLK,
            ENETB_RX_COL      => ENETB_RX_COL,
            ENETB_RX_CRS      => ENETB_RX_CRS,
            ENETB_RX_D        => ENETB_RX_D,
            ENETB_RX_DV       => ENETB_RX_DV,
            ENETB_RX_ER       => ENETB_RX_ER,
            ENETB_GTX_CLK     => ENETB_GTX_CLK,
            ENETB_TX_D        => ENETB_TX_D,
            ENETB_TX_EN       => ENETB_TX_EN,
            ENETB_TX_ER       => ENETB_TX_ER,
            ENETB_LED_LINK100 => ENETB_LED_LINK100,

            ------ MDIO to PHY ------
            ENET_MDC  => ENET_MDC,
            ENET_MDIO => ENET_MDIO
      );
end tb;