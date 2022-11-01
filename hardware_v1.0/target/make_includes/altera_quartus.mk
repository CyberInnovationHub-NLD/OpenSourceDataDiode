################################################################################
##
## (C) COPYRIGHT 2006-2014 TECHNOLUTION BV, GOUDA NL
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
## This is the Altera Quartus build make file. It is based on the makefile
## provided by Altera.
################################################################################


################################################################################
## Makefile settings
################################################################################

# Delete all known suffixes.
.SUFFIXES:

# Avoid removal of intermediate targets.
.SECONDARY:

# Do not delete report files in case make is interrupted.
.PRECIOUS: $(REPORT_FILE_DIR)/%.rpt

# Avoid parallel builds.
.NOTPARALLEL:


################################################################################
## Tool variables
################################################################################

# Default options.
QUARTUS_64BIT ?= 1
IGNORE_TIMING_PIN_ERRORS ?= no
TIMEQUEST ?= yes
ALLOW_TIME_LIMITED ?= false

# Setup tool environments.
TL_ENV ?= true
ENVIRONMENTS := quartus/$(QUARTUS_VERSION)

QUARTUS_ASM = ${TL_ENV} $(ENVIRONMENTS) && quartus_asm
QUARTUS_CDB = ${TL_ENV} $(ENVIRONMENTS) && quartus_cdb
QUARTUS_FIT = ${TL_ENV} $(ENVIRONMENTS) && quartus_fit
QUARTUS_GUI = ${TL_ENV} $(ENVIRONMENTS) && quartus
QUARTUS_MAP = ${TL_ENV} $(ENVIRONMENTS) && quartus_map
QUARTUS_SH  = ${TL_ENV} $(ENVIRONMENTS) && quartus_sh
QUARTUS_STA = ${TL_ENV} $(ENVIRONMENTS) && quartus_sta
QUARTUS_TAN = ${TL_ENV} $(ENVIRONMENTS) && quartus_tan

export QUARTUS_64BIT

MAP_ARGS += --write_settings_files=off
FIT_ARGS += --write_settings_files=off
ASM_ARGS += --write_settings_files=off
TAN_ARGS +=
STA_ARGS +=
CDB_ARGS +=


################################################################################
## Global variables
################################################################################

WORK_DIR		?= work
QUARTUS_PROJECT_FILE	?= $(PROJECT).qpf
QUARTUS_SETTINGS_FILE	?= $(PROJECT).qsf

# Get some settings from the Quartus Settings File...
VHDL_FILES			= $(shell grep "set_global_assignment -name VHDL_FILE" $(QUARTUS_SETTINGS_FILE) \
					| fromdos | awk '{print $$4}' | tr -d \" | tr "\n" " ")
QSYS_FILES			= $(shell grep "set_global_assignment -name QSYS_FILE" $(QUARTUS_SETTINGS_FILE) \
					| fromdos | awk '{print $$4}' | tr -d \" | tr "\n" " ")
SDC_FILES			= $(shell grep "set_global_assignment -name SDC_FILE" $(QUARTUS_SETTINGS_FILE) \
					| fromdos | awk '{print $$4}' | tr -d \" | tr "\n" " ")
TIMEQUEST_REPORT_SCRIPTS	= $(shell grep "set_global_assignment -name TIMEQUEST_REPORT_SCRIPTS" $(QUARTUS_SETTINGS_FILE) \
					| fromdos | awk '{print $$4}' | tr -d \" | tr "\n" " ")
MISC_FILES			= $(shell grep "set_global_assignment -name MISC_FILE" $(QUARTUS_SETTINGS_FILE) \
					| fromdos | awk '{print $$4}' | tr -d \" | tr "\n" " ")

# We only support WORK_DIR == PROJECT_OUTPUT_DIRECTORY
PROJECT_OUTPUT_DIRECTORY = $(shell grep "^set_global_assignment -name PROJECT_OUTPUT_DIRECTORY" $(QUARTUS_SETTINGS_FILE) \
	| fromdos | awk '{print $$4}' | tr -cd "[:print:]")
ifneq ($(PROJECT_OUTPUT_DIRECTORY),$(WORK_DIR))
    $(error please set PROJECT_OUTPUT_DIRECTORY to ${WORK_DIR} ('set_global_assignment -name PROJECT_OUTPUT_DIRECTORY work' in qsf))
endif

SOURCE_FILES = $(VHDL_FILES) $(QSYS_FILES) $(MISC_FILES)

INPUT_DEPENDS		?= 

################################################################################
## Includes
################################################################################

include $(GLOBAL_INCS)/shell.inc
THIS_MAKE_DEP := $(THIS_MAKE_DEP) $(lastword $(MAKEFILE_LIST))

################################################################################
## Common rules
############################################################################

# Use the cat function to break when the build reports are not found
.PHONY: $(PROJECT).qflow
$(PROJECT).qflow: $(WORK_DIR) $(INPUT_DEPENDS)
	@$(call print_cmd_info, "QSH FLOW - COMPILE", $(PROJECT))
	@$(QUARTUS_SH) --flow compile $(PROJECT) > $(WORK_DIR)/$(OUTPUT_MASTER_LOG)
	@cp $(WORK_DIR)/$(OUTPUT_MASTER_LOG) $(WORK_DIR)/$(@F).outlog
	@! grep "Critical Warning" $(PROJECT_OUTPUT_DIRECTORY)/$(PROJECT).*.rpt
	@cat $(PROJECT_OUTPUT_DIRECTORY)/$(PROJECT).map.rpt > /dev/null || cat $(PROJECT_OUTPUT_DIRECTORY)/$(PROJECT).syn.rpt > /dev/null
	@cat $(PROJECT_OUTPUT_DIRECTORY)/$(PROJECT).fit.rpt > /dev/null
	@cat $(PROJECT_OUTPUT_DIRECTORY)/$(PROJECT).sta.rpt > /dev/null
	@$(call print_cmd_info, "QSH FLOW - DONE", $(PROJECT))

.PHONY: $(PROJECT).timing
$(PROJECT).timing : $(PROJECT).qflow

$(PROJECT_OUTPUT_DIRECTORY)/$(PROJECT).sof : $(PROJECT).qflow


################################################################################
## Short target names
################################################################################

.PHONY: map fit asm tan
map: $(PROJECT).qflow
fit: $(PROJECT).qflow
asm: $(PROJECT).qflow
tan: $(PROJECT).qflow

$(WORK_DIR):
	@$(call print_cmd_info, "MK WORK DIR", $@)
	@$(MKDIR) -p $@

OUTPUT_MASTER_LOG = $(PROJECT).master.outlog

    ############################################################################
    ## Last Log
    ############################################################################
    ## Shows the output of the last log (this is the content of the master log).
    ## Can be used in case of an error.
    ############################################################################

.PHONY: ll
ll:
	@$(call print_cmd_info, "CAT LAST LOG", "Shows the last log")
	@cat $(WORK_DIR)/$(OUTPUT_MASTER_LOG)


    ############################################################################
    ## Tail Master Log
    ############################################################################
    ## Follows the output of the master log.
    ############################################################################

.PHONY: tml
tml:
	@$(call print_cmd_info, "TAIL MASTER LOG", "Shows the last lines of the master log")
	@tail -F $(WORK_DIR)/$(OUTPUT_MASTER_LOG)


    ############################################################################
    ## Project initialization
    ############################################################################

$(QUARTUS_PROJECT_FILE) $(QUARTUS_SETTINGS_FILE):
	@$(call print_cmd_info, "PREPARE", "Preparing project.")
	$(QUARTUS_SH) --prepare $(PROJECT)


    ############################################################################
    ## Clean
    ############################################################################
    ## Commands to clean the environment
    ############################################################################

.PHONY: altera-clean
altera-clean:
	@$(call print_cmd_info, "CLEANUP", "altera environment")
	-@$(RM) *.chg
	-@$(RM) *.db_info
	-@$(RM) *.done
	-@$(RM) *.eqn
	-@$(RM) *.htm
	-@$(RM) *.jdi
	-@$(RM) *.pin
	-@$(RM) *.pincheck
	-@$(RM) *.pof
	-@$(RM) *.qws
	-@$(RM) *.rbf
	-@$(RM) *.rpt
	-@$(RM) *.smlog
	-@$(RM) *.smsg
	-@$(RM) *.sof
	-@$(RM) *.sopcinfo
	-@$(RM) *.summary
	-@$(RM) *.timing
	-@$(RM) *.ttf
	-@$(RM) PLLJ_PLLSPE_INFO.txt
	-@$(RM) a5_pin_model_dump.txt
	-@$(RM) c5_pin_model_dump.txt
	-@$(RM) cmp_state.ini
	-@$(RM) *.master.outlog
	-@$(RM) *_summary.csv
	-@$(RMDIR) db
	-@$(RMDIR) hc_output
	-@$(RMDIR) incremental_db
	-@$(RMDIR) hps_isw_handoff
	-@$(RMDIR) qdb
	-@$(RMDIR) tmp-clearbox
	-@$(RMDIR) tl_git_info*
	-@$(RMDIR) $(WORK_DIR)


    ############################################################################
    ## Quartus II GUI
    ############################################################################
    ## Commands to start the GUI.
    ############################################################################

.PHONY: altera-gui
altera-gui: $(QUARTUS_PROJECT_FILE)
	@$(call print_cmd_info, "GUI", "Quartus II")
	@$(QUARTUS_GUI) $(QUARTUS_PROJECT_FILE) &


################################################################################
## Short target names
################################################################################

    ############################################################################
    ## Default output target (project.sof)
    ############################################################################

.PHONY: sof_file
sof_file: $(PROJECT_OUTPUT_DIRECTORY)/$(PROJECT).sof


    ############################################################################
    ## Output rule for time-limited version.
    ############################################################################
    ## Can be used to (re)build an output file when not all required licenses
    ## are available. When using the default rule the last step is done every
    ## time because the target file is missing. This rule avoids that. It also
    ## allows a build to succeed, while the default rule will fail if a
    ## time-limited file is generated.
    ############################################################################

.PHONY: time_limited
time_limited: ALLOW_TIME_LIMITED=true
time_limited: $(PROJECT_OUTPUT_DIRECTORY)/$(PROJECT)_time_limited.sof


    ############################################################################
    ## Build information
    ############################################################################
    ## Displays all possible build targets
    ############################################################################

.PHONY: altera-info
altera-info:
	@$(ECHO) "*********************************************************"
	@$(ECHO) "*** ALTERA TARGETS"
	@$(ECHO) "*********************************************************"
	@$(ECHO) "*** File targets"
	@$(ECHO) "***   map                   : synthesis & target mapping"
	@$(ECHO) "***   fit                   : place and route"
	@$(ECHO) "***   asm                   : generates a target image"
	@$(ECHO) "***   <target>.sof          : generates a sof image"
	@$(ECHO) "*** Virtual targets"
	@$(ECHO) "***   <target>.timing       : (re-)generates timing report"
	@$(ECHO) "***   	"
	@$(ECHO) "***   altera-clean          : cleans all files that are generated by the altera makefile"
	@$(ECHO) "***   altera-info           : this info"
	@$(ECHO) "***   <target>.tml          : shows the output of the master log"
	@$(ECHO) "***   <target>.ll           : shows last log, use to show the log of the"
	@$(ECHO) "***   			            last command incase of an error"
	@$(ECHO) "*********************************************************"
