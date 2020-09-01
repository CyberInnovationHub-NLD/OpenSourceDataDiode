###############################################################################
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
###############################################################################
## This is the Altera Quartus project configuration file. All project settings
## are found in the .qpf and .qsf files.
###############################################################################
## NOTE: The following restrictions apply when using this make file:
##		- Timing check only supported for FLOW type 'make'.
###############################################################################

## GLOBAL_INCS should point to the make include files directory. This value
## should only be overriden when it is not detected automatically.
export GLOBAL_INCS = make_includes

.SUFFIXES: # Delete all known suffixes.


###############################################################################
## Project configuration
###############################################################################
## Specify the Quartus II version and the the name of the design (project).
## NOTE: The Quartus II project and settings files (*.qpf and *.qsf,
## respectively) must match the name of the project.
###############################################################################

QUARTUS_VERSION		= 16.1.2

TARGET	 		= OSDD_max10

PROJECT			:= $(TARGET)


###############################################################################
## Additional build options
###############################################################################

## FLOW (make/flow)
## Set to "make" to use make to handle the process flow.
## Set to "flow" to use the Quartus flow command to handle the process flow.
FLOW = flow

## QUARTUS_64BIT (1/0, default 1)
## Set to "1" to use 64-bit processing in Quartus (required for some FPGA
## architectures).
## Set to "0" to use 32-bit processing in Quartus.
QUARTUS_64BIT = 1

## TIMEQUEST (yes/no, default yes)
## Set to "yes" to use the TimeQuest Timing Analyzer.
## Set to "no" (or any other value) to use the Classic Timing Analyzer.
TIMEQUEST = yes

## IGNORE_TIMING_PIN_ERRORS (yes/no, default no)
## When this variable is set to set yes a bit file is generated even when the
## timing constraints are not met or not all pins in the design are assigned.
##
## NOTE: Timing check only supported when FLOW is set to 'make'.
IGNORE_TIMING_PIN_ERRORS = no

## ALLOW_TIME_LIMITED (true/false, default false)
## Set to "true" to allow time-limited .sof files to be generated.
ALLOW_TIME_LIMITED = false

###############################################################################
## Set command arguments (advanced)
###############################################################################

MAP_ARGS +=
FIT_ARGS +=
ASM_ARGS +=
TAN_ARGS +=
STA_ARGS +=
CDB_ARGS +=

###############################################################################
## Default rules
###############################################################################

all: sof_file

clean: altera-clean

gui: altera-gui

###############################################################################
#### WARNING!!
###############################################################################
#### Do not touch the settings below (these statements should be placed at the
#### end of the makefile)
###############################################################################

# The following variable contains the actual rule file
RULES_INCLUDE = altera_quartus.mk


###############################################################################
#### Automatically find make_include directory
###############################################################################

CUR_LEV := $(subst /, , $(subst ' ',_, $(CURDIR)))
LEVELS := $(wordlist 3, $(words $(CUR_LEV)),$(CUR_LEV))
$(foreach level, $(LEVELS), $(eval LE := $(LE)../) $(eval MAKE_INCLUDE_SEARCH_PATH := $(MAKE_INCLUDE_SEARCH_PATH) $(LE)make_includes))
GLOBAL_INCS_REL := $(firstword $(wildcard $(MAKE_INCLUDE_SEARCH_PATH)))
GLOBAL_INCS ?= $(CURDIR)/$(GLOBAL_INCS_REL)

export GLOBAL_INCS


###############################################################################
#### Include simulation rules
###############################################################################

RULES_INCLUDE_EXISTS := $(shell [[ -e $(GLOBAL_INCS)/$(RULES_INCLUDE) ]] && echo "ok")

ifneq ($(RULES_INCLUDE_EXISTS),ok)
$(error $(RULES_INCLUDE) include file not found. Please enter path to the make_includes directory manually)
endif

include $(GLOBAL_INCS)/$(RULES_INCLUDE)
