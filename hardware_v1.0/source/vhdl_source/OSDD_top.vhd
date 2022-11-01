-------------------------------------------------------------------------------
-- Title      : OSDD_top
-------------------------------------------------------------------------------
-- Description: top level for MAX10 development kit
-- Written for "Open source data diode (OSDD)" project
-------------------------------------------------------------------------------
library ieee;
use ieee.std_logic_1164.all;
use ieee.numeric_std.all;

library work;

library altera_mf;
use altera_mf.altera_mf_components.all;
library GPIO_DDIO_buf;
library GPIO_DDIO_out;

entity OSDD_top is
      port (
            CLK_LVDS_125_P : in std_logic;
            FPGA_RESETN    : in std_logic;
            USER_LED       : out std_logic_vector(4 downto 0);

            ------ PHY_A ------
            ENETA_INTN   : in std_logic;
            ENETA_RESETN : out std_logic;

            ENETA_RX_CLK : in std_logic;
            ENETA_RX_COL : in std_logic;
            ENETA_RX_CRS : in std_logic;
            ENETA_RX_D   : in std_logic_vector(3 downto 0);
            ENETA_RX_DV  : in std_logic;
            ENETA_RX_ER  : in std_logic;

            ENETA_GTX_CLK     : out std_logic;
            ENETA_TX_CLK      : out std_logic;
            ENETA_TX_D        : out std_logic_vector(3 downto 0); --
            ENETA_TX_EN       : out std_logic;
            ENETA_TX_ER       : out std_logic; --
            ENETA_LED_LINK100 : out std_logic;

            ------ PHY_B ------
            ENETB_INTN   : in std_logic;
            ENETB_RESETN : out std_logic;

            ENETB_RX_CLK : in std_logic;
            ENETB_RX_COL : in std_logic;
            ENETB_RX_CRS : in std_logic;
            ENETB_RX_D   : in std_logic_vector(3 downto 0);
            ENETB_RX_DV  : in std_logic;
            ENETB_RX_ER  : in std_logic;

            ENETB_GTX_CLK     : out std_logic;
            ENETB_TX_CLK      : out std_logic;
            ENETB_TX_D        : out std_logic_vector(3 downto 0); --
            ENETB_TX_EN       : out std_logic;
            ENETB_TX_ER       : out std_logic; --
            ENETB_LED_LINK100 : out std_logic;

            ------ MDIO to PHY ------
            ENET_MDC  : out std_logic;  -- v
            ENET_MDIO : inout std_logic -- v
      );
end OSDD_top;

architecture arch of OSDD_top is
      --reset and clock signals
      signal clk_125 : std_logic := '0';
      signal clk_25  : std_logic := '0';
      signal clk_2_5 : std_logic := '0';

      signal reset_in, reset_pll, reset_phy_a, reset_phy_b : std_logic;
      signal reset_sync1, reset_sync2                      : std_logic;
      signal speed_stat_phy_0, speed_stat_phy_1            : std_logic_vector(1 downto 0) := "11";

      -- PHY_A signals
      signal phya_tx_clk      : std_logic;
      signal phya_rx_vec      : std_logic_vector(9 downto 0) := (others => '0');
      signal phya_rx_vec_reg  : std_logic_vector(9 downto 0) := (others => '0');
      signal phya_rx_vec_reg2 : std_logic_vector(9 downto 0) := (others => '0');
      signal phya_tx_vec      : std_logic_vector(9 downto 0) := (others => '0');

      -- PHY_B signals
      signal phyb_tx_clk      : std_logic;
      signal phyb_rx_vec      : std_logic_vector(9 downto 0) := (others => '0');
      signal phyb_rx_vec_reg  : std_logic_vector(9 downto 0) := (others => '0');
      signal phyb_rx_vec_reg2 : std_logic_vector(9 downto 0) := (others => '0');
      signal phyb_tx_vec      : std_logic_vector(9 downto 0) := (others => '0');

      --mdio
      signal mdio_i   : std_logic := '0';
      signal mdio_o   : std_logic := '0';
      signal mdio_t   : std_logic := '0';
      signal mdio_mdc : std_logic := '0';

begin
      process (CLK_LVDS_125_P)
      begin
            if (rising_edge(CLK_LVDS_125_P)) then
                  reset_sync1 <= not(FPGA_RESETN);
                  reset_sync2 <= reset_sync1;
                  reset_in    <= reset_sync2;
            end if;
      end process;
      ----------------------------------------------------
      -- PHYA: mapping                                  --
      ----------------------------------------------------
      ENETA_GTX_CLK     <= '0';
      ENETA_TX_CLK      <= '0';
      ENETA_TX_D        <= "0000";
      ENETA_TX_EN       <= '0';
      ENETA_LED_LINK100 <= '0';
      ------ RGMII mapping ------
      process (ENETA_RX_CLK)
      begin
            if (rising_edge(ENETA_RX_CLK)) then
                  phya_rx_vec                 <= phya_rx_vec_reg2;
                  phya_rx_vec_reg2            <= phya_rx_vec_reg;
                  phya_rx_vec_reg(4 downto 0) <= ENETA_RX_DV & ENETA_RX_D;
            elsif (falling_edge(ENETA_RX_CLK)) then
                  phya_rx_vec_reg(9 downto 5) <= ENETA_RX_DV & ENETA_RX_D;
            end if;
      end process;
      ------- Connect clock and reset outputs. ------
      ENETA_TX_ER  <= '0';
      ENETA_RESETN <= not(reset_phy_a);

      ----------------------------------------------------
      -- PHYB: mapping                                  --
      ----------------------------------------------------
      ENETB_TX_CLK      <= '0';
      ENETB_LED_LINK100 <= '0';
      ------ TX ------
      PHYB_OUT_DATA : entity GPIO_DDIO_out.GPIO_DDIO_out
            port map(
                  outclock            => phyb_tx_clk,
                  din(11)             => '0',
                  din(10)             => phyb_tx_vec(9),
                  din(9 downto 6)     => phyb_tx_vec(8 downto 5),
                  din(5)              => '1',
                  din(4)              => phyb_tx_vec(4),
                  din(3 downto 0)     => phyb_tx_vec(3 downto 0),
                  pad_out(5)          => ENETB_GTX_CLK,
                  pad_out(4)          => ENETB_TX_EN,
                  pad_out(3 downto 0) => ENETB_TX_D
            );

      ------- Connect clock and reset outputs. ------
      ENETB_TX_ER  <= '0';
      ENETB_RESETN <= not(reset_phy_b);

      ----------------------------------------------------
      -- MDIO                                           --
      ----------------------------------------------------
      mdio_i   <= ENET_MDIO;
      ENET_MDC <= mdio_mdc;
      -- Convert MDIO
      ENET_MDIO <= mdio_o when mdio_t = '0' else
            'Z';

      ----------------------------------------------------
      -- Ethernet interface handeling                   --
      ----------------------------------------------------
      i_eth0 : entity work.OSDD_ethernet
            generic map(
                  g_clock_frequency => 125000000, -- system clock
                  g_mdc_frequency   => 2500000,   -- mdio clock
                  g_phya_addr       => "00000",
                  g_phyb_addr       => "00001",
                  g_shared_mdiobus  => true --if false: both phys have address a
            )
            port map(
                  clk         => CLK_LVDS_125_P,
                  phya_rx_clk => ENETA_RX_CLK,
                  phyb_tx_clk => phyb_tx_clk,
                  reset       => reset_in,

                  reset_phy_a => reset_phy_a,
                  reset_phy_b => reset_phy_b,

                  phya_rx_vec => phya_rx_vec,
                  phyb_tx_vec => phyb_tx_vec,

                  interrupt_phy_0 => not(ENETA_INTN),
                  interrupt_phy_1 => not(ENETB_INTN),

                  mdio_i   => mdio_i,
                  mdio_o   => mdio_o,
                  mdio_t   => mdio_t,
                  mdio_mdc => mdio_mdc,

                  speed_stat_phy_0_d => speed_stat_phy_0,
                  speed_stat_phy_1_d => speed_stat_phy_1
            );
      USER_LED(0)          <= not((ENETA_INTN)) or not(ENETB_INTN);
      USER_LED(2 downto 1) <= not(speed_stat_phy_0);
      USER_LED(4 downto 3) <= not(speed_stat_phy_1);
end arch;