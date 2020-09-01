################################################################################
##
## (C) COPYRIGHT 2004-2013 TECHNOLUTION BV, GOUDA NL
## | =======          I                   ==          I    =
## |    I             I                    I          I
## |    I   ===   === I ===  I ===   ===   I  I    I ====  I   ===  I ===
## |    I  /   \ I    I/   I I/   I I   I  I  I    I  I    I  I   I I/   I
## |    I  ===== I    I    I I    I I   I  I  I    I  I    I  I   I I    I
## |    I  \     I    I    I I    I I   I  I  I   /I  \    I  I   I I    I
## |    I   ===   === I    I I    I  ===  ===  === I   ==  I   ===  I    I
## |                 +---------------------------------------------------+
## +----+            |  +++++++++++++++++++++++++++++++++++++++++++++++++|
##      |            |             ++++++++++++++++++++++++++++++++++++++|
##      +------------+                          +++++++++++++++++++++++++|
##                                                         ++++++++++++++|
##                                                                  +++++|
##
################################################################################
## cocoTB Simulation include file for GHDL
################################################################################
## Based on the great work of Potential Ventures Ltd
################################################################################

################################################################################
## Makefile settings
################################################################################
# Delete all known suffixes.
.SUFFIXES:

# Avoid removal of intermediate targets.
.SECONDARY:


MKFILE_PATH := $(abspath $(lastword $(MAKEFILE_LIST)))
CUR_MK_DIR := $(dir $(MKFILE_PATH))

################################################################################
## include cocotb build system
################################################################################
TL_ENV ?= true
ENVIRONMENTS ?= cocotb-modelsim

ifndef COCOTB
  COCOTB 		:= $(shell $(TL_ENV) $(ENVIRONMENTS) && echo $$COCOTB)
endif
ifndef COCOTB_LIBDIR
  COCOTB_LIBDIR 	:= $(shell $(TL_ENV) $(ENVIRONMENTS) && echo $$COCOTB_LIBDIR)
endif
ifndef COCOTB_LIBEXT
  COCOTB_LIBEXT		:= $(shell $(TL_ENV) $(ENVIRONMENTS) && echo $$COCOTB_LIBEXT)
endif
ifndef COCOTB_VERSION
  COCOTB_VERSION	:= $(shell $(TL_ENV) $(ENVIRONMENTS) && echo $$COCOTB_VERSION)
endif

include $(COCOTB)/makefiles/Makefile.inc

# make sure color is maintained when using tee
export COCOTB_ANSI_OUTPUT = 1
ifndef V
export COCOTB_REDUCED_LOG_FMT = 1
HIDE=@
else
HIDE=
endif

OUT_HANDLER				= $(if $(OUT_VERBOSE), | tee -i , >)

################################################################################
## set directories and source files
################################################################################

WORK_DIR		?= $(CURDIR)/work
SIM_BUILD		?= $(WORK_DIR)/sim_build
RUN_DIR			?= $(WORK_DIR)/run

VHDL_SOURCES 	:= $(abspath $(VHDL_SOURCES))

CUSTOM_SIM_DEPS	?= $(wildcard $(patsubst %,%/*py,$(subst :, , $(PYTHONPATH)))) Makefile

TOPLEVEL_LANG = verilog

################################################################################
## variables
################################################################################
RTL_LIBRARY ?= work

.NOTPARALLEL:

# if modules is not set, use the module
ifndef MODULES
MODULES = $(MODULE)
$(MODULE)TOPLEVEL = $(TOPLEVEL)
$(MODULE)TESTCASE = $(TESTCASE)
$(MODULE)RUN_TIME = $(RUNTIME)
$(MODULE)DO_GUI_OVERRIDE = $(DO_GUI_OVERRIDE)
$(MODULE)DO_GUI_BEFORE = $(DO_GUI_BEFORE)
$(MODULE)DO_BEFORE = $(DO_BEFORE)
$(MODULE)DO_AFTER = $(DO_AFTER)
$(MODULE)RUN_TIME = $(RUN_TIME)
$(MODULE)SIM_ARGS = $(SIM_ARGS)

gui: $(MODULE).gui
endif

$(foreach module,$(MODULES),$(eval $(module).vc: $($(module)TOPLEVEL).vc))
$(foreach module,$(MODULES),$(eval $(module): $(module)_check))
CHECK_LIST = $(patsubst %, %_check, $(MODULES))
RESULT_LIST = $(patsubst %, $(RUN_DIR)/%_sim.log, $(MODULES))
GUI_LIST = $(patsubst %, %.gui, $(MODULES))

$(foreach module,$(MODULES),$(eval $(module)PY_MODULE ?= $(module)))

################################################################################
## simulation phony rules
################################################################################
.PHONY: analyse sim regression check clean info-modules

sim:
	$(HIDE)rm -rf $(RESULT_LIST) && OUT_VERBOSE=1 $(MAKE) regression

%.sim:
	$(HIDE)rm -f $(RUN_DIR)/$*_sim.log && OUT_VERBOSE=1 $(MAKE) $*

analyse: $(SIM_BUILD)/$(TOPLEVEL)

info:
	@echo $(TOPLEVELS)
	@echo $(CHECK_LIST)
	@echo $(WORK_DIR)
	@echo $(AUTO_DEP_PATHS)

info-modules::
	@$(ECHO) "*****************************************************************************************"
	@$(ECHO) "*** Simulation modules"
	@$(ECHO) "*****************************************************************************************"
	@$(foreach module,$(MODULES),$(ECHO) "***   $(module)";)
	@$(ECHO) "*****************************************************************************************"

# Regression rule uses Make dependencies to determine whether to run the simulation
regression: $(CHECK_LIST)
%_check: $(WORK_DIR)/run/%_sim.log
	@$(call print_cmd_info_nonl,"CHECK","$(*F)")
	$(HIDE)grep -E "**[ ]+ERRORS : 0[ ]+**" $< > /dev/null && echo " --> success" || (echo " --> failed" && false)


clean:: vsim-clean
	@$(call print_cmd_info,"CLEAN","$(WORK_DIR)")
	-$(HIDE)rm -rf $(WORK_DIR)
	-$(HIDE)rm -rf dataset.asdb

clean-sim::
	@$(call print_cmd_info,"CLEAN","$(SIM_BUILD)")
	-$(HIDE)rm -rf $(SIM_BUILD)

clean-run::
	@$(call print_cmd_info,"CLEAN","$(RUN_DIR)")
	-$(HIDE)rm -rf $(RUN_DIR)

################################################################################
## riviera cocotb rules
################################################################################
$(SIM_BUILD) $(RUN_DIR):
	@$(call print_cmd_info,"MKDIR","$@")
	$(HIDE)mkdir -p $@

# create the DO file
%.tc.do: $(CONFIG_FILES) $(CUSTOM_SIM_DEPS)
	@$(call print_cmd_info,"VSIM DO GEN",$(@))
	$(if $($(*F)DO_OVERRIDE),\
		@echo $($(*F)DO_OVERRIDE) > $(VWORK_DIR)/$(@F),\
		@echo "onerror {resume}; onbreak {resume}; $($(*F)DO_BEFORE); \
		run $($(*F)RUN_TIME); $($(*F)DO_AFTER); exit;"\
		> $(VWORK_DIR)/$(@F)\
	 )
	 
# create the DO file
%.gui.do: $(CONFIG_FILES) $(CUSTOM_SIM_DEPS)
	@$(call print_cmd_info,"VSIM GUI DO GEN",$(@))
	$(if $($(*F)DO_GUI_OVERRIDE),\
		@echo $($(*F)DO_GUI_OVERRIDE) > $(VWORK_DIR)/$(@F),\
		@echo 'onerror {break}; onbreak {break}; \
			proc save_range {} {global last_range; set last_range [wv.zoom.dump]; wv.cursors.dump -onlyactive > ./work/activecursor.xml; list}; \
			proc load_range {} {global last_range; set endpoints [split $$last_range "-"]; wv.zoom.range -from [lindex $$endpoints 0] -to [lindex $$endpoints 1]; set cursor_fp [open ./work/activecursor.xml]; set cursor_xml [read $$cursor_fp]; close $$cursor_fp; regexp {value="([^"]*)"} $$cursor_xml cursor_match cursor_position; wv.cursors.add -time $$cursor_position; wv.cursors.removeall; list}; \
			' "proc remake     {} {exec make $($(*F)TOPLEVEL).vc}; \
			proc run_once   {} {$($(*F)DO_GUI_BEFORE); run $($(*F)RUN_TIME); $($(*F)DO_GUI_AFTER)}; \
			proc rerun      {} {save_range; remake; restart; run_once; load_range}; \
			run_once"\
		> $(VWORK_DIR)/$(@F)\
	 )

$(RUN_DIR)/%_sim.log: %.tc.do %.vc $(CUSTOM_SIM_DEPS) | $(RUN_DIR)
	@$(call print_cmd_info,"RUN ", $(*F))
	$(HIDE)-rm -f $(SIM_BUILD)/$@
	$(HIDE)$(TL_ENV) $(ENVIRONMENTS) && \
		PYTHONPATH=$(COCOTB_LIBDIR):$(COCOTB):$(PWD):$(PYTHONPATH) \
		LD_LIBRARY_PATH=$(COCOTB_LIBDIR):$${LD_LIBRARY_PATH} \
		TL_IMPORT_LIBS=$(TL_IMPORT_LIBS) \
		MODULE=$($(*F)PY_MODULE) \
		TESTCASE=$($(*F)TESTCASE) \
		TOPLEVEL=$($(*F)TOPLEVEL) \
		TOPLEVEL_LANG=$(TOPLEVEL_LANG) \
		vsim -c -loadvhpi libvhpi:vhpi_startup_routines_bootstrap -O2 -do $(WORK_DIR)/$(*F).tc.do $($(*F)SIM_ARGS) $($(*F)TOPLEVEL) $(OUT_HANDLER) $(RUN_DIR)/$(*F)_sim.log 2>&1

%.gui: %.gui.do %.vc $(CUSTOM_SIM_DEPS) | $(RUN_DIR)
	@$(call print_cmd_info,"RUN GUI ", $(*F))
	$(HIDE)-rm -f $(SIM_BUILD)/$@
	$(HIDE)$(TL_ENV) $(ENVIRONMENTS) && \
		PYTHONPATH=$(COCOTB_LIBDIR):$(COCOTB):$(PWD):$(PYTHONPATH) \
		LD_LIBRARY_PATH=$(COCOTB_LIBDIR):$${LD_LIBRARY_PATH} \
		TL_IMPORT_LIBS=$(TL_IMPORT_LIBS) \
		MODULE=$($(*F)PY_MODULE) \
		TESTCASE=$($(*F)TESTCASE) \
		TOPLEVEL=$($(*F)TOPLEVEL) \
		TOPLEVEL_LANG=$(TOPLEVEL_LANG) \
		COCOTB_ANSI_OUTPUT=0 \
		vsim -gui -loadvhpi libvhpi:vhpi_startup_routines_bootstrap -interceptcoutput -O2 -do $(WORK_DIR)/$(*F).gui.do $($(*F)SIM_ARGS) $($(*F)TOPLEVEL) $(OUT_HANDLER) $(RUN_DIR)/$(*F)_sim.log &

include $(CUR_MK_DIR)/shell.inc

