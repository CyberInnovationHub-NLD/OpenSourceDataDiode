set_global_assignment -entity "GPIO_DDIO_out" -library "lib_GPIO_DDIO_out" -name IP_TOOL_NAME "altera_gpio_lite"
set_global_assignment -entity "GPIO_DDIO_out" -library "lib_GPIO_DDIO_out" -name IP_TOOL_VERSION "16.1"
set_global_assignment -entity "GPIO_DDIO_out" -library "lib_GPIO_DDIO_out" -name IP_TOOL_ENV "mwpim"
set_global_assignment -library "lib_GPIO_DDIO_out" -name SPD_FILE [file join $::quartus(sip_path) "GPIO_DDIO_out.spd"]

set_global_assignment -library "lib_GPIO_DDIO_out" -name MISC_FILE [file join $::quartus(sip_path) "GPIO_DDIO_out_sim/GPIO_DDIO_out.vhd"]
set_global_assignment -library "lib_GPIO_DDIO_out" -name MISC_FILE [file join $::quartus(sip_path) "GPIO_DDIO_out_sim/altera_gpio_lite/altera_gpio_lite.sv"]
set_global_assignment -library "lib_GPIO_DDIO_out" -name MISC_FILE [file join $::quartus(sip_path) "GPIO_DDIO_out_sim/altera_gpio_lite/mentor/altera_gpio_lite.sv"]
