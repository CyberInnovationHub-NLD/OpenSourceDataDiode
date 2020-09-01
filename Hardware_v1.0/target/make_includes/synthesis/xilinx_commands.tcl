###############################################################################
## (C) COPYRIGHT 2009-2011 TECHNOLUTION BV, GOUDA NL
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
###############################################################################
## Title      : Xilinx Build Command handler
## Author     : Sijmen Woutersen <sijmen.woutersen@technolution.eu>
###############################################################################
## Description: This scripts contains all supported xilinx toolchain commands
###############################################################################

# build settings, no not modify! use functions to alter them from the project file
set protected 1
set no_pinning_check 0
set no_timing_check 0
set attempts 1

# set amount of map/par attempts
proc set_attempts {val} {
    global attempts
    set attempts $val
}

# disable pin locked check
proc disable_pinning_check {} {
    global no_pinning_check
    set no_pinning_check 1
    log_warning "Pinning check disabled"
}

# disable timing check
proc disable_timing_check {} {
    global no_timing_check
    set no_timing_check 1
    log_warning "Timing check disabled"
}

# disable error catching, prints detailed errors but may currupt project file
proc disable_protection {} {
    global protected
    set protected 0
}

# check if the timing constraints where met
proc timing_check {} {
    global top_entity
    global no_timing_check

    if {$no_timing_check} {
        log_warning "Timing check disabled"
        return 1
    }

    if [is_cpld] {
        set fp [open $top_entity.tim r]
        set twr_data [read $fp]
        close $fp

        if {[regexp "Not Met" $twr_data] != 0} {
            return 0
        } else {
            return 1
        }
    } else {
        set fp [open $top_entity.twr r]
        set twr_data [read $fp]
        close $fp

        if {[regexp "All constraints were met." $twr_data] == 0} {
            return 0
        } else {
            return 1
        }
    }
}
# check if all pins are assigned
proc pinning_check {} {
    global top_entity
    global no_pinning_check

    if {$no_pinning_check} {
        log_warning "Pinning check disabled"
        return 1
    }

    if [is_cpld] {
        log_warning "pinning is not checked for CPLDs"
        return 1
    } else {
        # par based data (this no longer works for newer ise versions, unknown which version is the first "newer" version)
        set fp [open $top_entity.par r]
        set par_data [read $fp]
        close $fp

        if {[regexp "Only a subset of IOs are locked." $par_data] == 0} {
            # check pad report below
        } else {
            log_error "Not all pins are assigned (based on .par)!"
            return 0
        }

        # pad report based data (may not work with older ISE versions, unknown at this point)
        set fp [open $top_entity.pad r]
        set pad_data [read $fp]
        close $fp

        if {[regexp "UNLOCATED" $pad_data] == 0} {
            return 1
        } else {
            log_error "Not all pins are assigned (based on .pad)!"
            return 0
        }
    }
}

# cleanup the build environment
proc cleanup {} {
    global work_dir

    # goto startup state
    global xil_state_startup
    switch_state $xil_state_startup

    log_info "Performing cleanup..."
    file delete -force $work_dir
    file delete -force xlnx_auto_0.ise
    file delete -force xlnx_auto_0_xdb
}

# build & analyze the design
proc build {tig} {
    global errors
    global top_entity
    global active_logfile
    global attempts
    global xil_state_project_implemented
    global xil_state_project_synthesized

    set done 0
    set attempt 0

    while {$done == 0 & $attempt < $attempts} {
        incr attempt

        # goto implemented state
        switch_state $xil_state_project_implemented

        if {$tig == 0} {
            log_info "Analyzing build ($attempt/$attempts)..."
            if [timing_check] {
                log_info "Timing constraints met"
                set done 1
            } else {
                if {$attempt == $attempts} {
                    log_error "Timing constraints NOT met"
                } else {
                    log_warning "Timing constraints not met, trying again..."
                    switch_state $xil_state_project_synthesized
                    project set "Starting Placer Cost Table (1-100)" "[expr [project get "Starting Placer Cost Table (1-100)" -process "Map"] + 1]" -process "Map"
                }
            }
        } else {
            set done 1
        }
    }

    if [pinning_check] {
        log_info "Pinning check ok"
    } else {
        log_error "Pinning check failed"
    }

    if {$errors == 0} {
        # goto bitfile generated state
        global xil_state_project_bitfile
        switch_state $xil_state_project_bitfile
    } else {
        log_error "Errors occured, not generating a bitfile"
    }
}

# synthesize only
proc synthesize_only {} {
    # goto synthesized state
    global xil_state_project_synthesized
    switch_state $xil_state_project_synthesized
}

# generate edif netlist
proc gen_edif {} {
    global top_entity

    # goto synthesized state
    global xil_state_project_synthesized
    switch_state $xil_state_project_synthesized

    # goto project closed state
    global xil_state_project_closed
    switch_state $xil_state_project_closed

    log_info "Generating EDIF netlist..."
    exec ngc2edif -w $top_entity.ngc $top_entity.ndf
}

# create project file
proc make_project {} {
    global work_dir
    global top_entity
    global ise_version_major

    # goto project closed state
    global xil_state_project_closed
    switch_state $xil_state_project_closed

    if [expr $ise_version_major >= 12] {
        set ise_ext "xise"
    } else {
        set ise_ext "ise"
    }

    # goto project open state
    global xil_state_project_synced
    switch_state $xil_state_project_synced

    # goto project closed state
    global xil_state_project_closed
    switch_state $xil_state_project_closed
}

# generate git info
proc gen_git_info {} {
    # goto project closed state
    global xil_state_project_closed
    switch_state $xil_state_project_closed

    # check if we can run gitinfo
    if {[info procs {gitinfo_xilinx}] != ""} {
        log_info "Generating Git Info..."
        gitinfo_xilinx ".."
    }
}


# start ise
proc start_ise {} {
    global work_dir
    global top_entity
    global ise_version_major

    # goto project closed state
    global xil_state_project_closed
    switch_state $xil_state_project_closed

    if [expr $ise_version_major >= 12] {
        set ise_ext "xise"
    } else {
        set ise_ext "ise"
    }

    # goto project open state
    global xil_state_project_synced
    switch_state $xil_state_project_synced

    # goto project closed state
    global xil_state_project_closed
    switch_state $xil_state_project_closed

    log_info "Starting ISE..."
    exec ise $top_entity.$ise_ext &
}

# start chipscope inserter
proc start_inserter {} {
    global work_dir
    global top_entity
    global device
    global speed
    global package
    global ise_version_major

    # goto project closed state (in work dir!)
    global xil_state_project_closed
    switch_state $xil_state_project_closed

    if [expr $ise_version_major >= 12] {
        set ins_ext ""
    } else {
        set ins_ext ".sh"
    }

    # create new cdc file if needed
    if {[file exists ../$top_entity.cdc] == 0} {
        log_info "Creating new chipscope file"
        exec inserter$ins_ext -create ../$top_entity.cdc
    }

    # goto synthesized state
    global xil_state_project_synthesized
    switch_state $xil_state_project_synthesized

    # goto project closed state
    global xil_state_project_closed
    switch_state $xil_state_project_closed

    log_info "Starting ChipScope Inserter..."
    exec inserter$ins_ext -edit ../$top_entity.cdc -p $device-$speed-$package -dd _ngo -i $top_entity.ngc $top_entity\_cs.ngc &
}

# start chipscope analyzer
proc start_analyzer {} {
    log_info "Starting ChipScope Analyzer..."
    exec analyzer.sh &
}

# start impact
proc start_impact {} {
    log_info "Starting Impact..."
    exec impact &
}

# start fpga editor
proc start_fpga_editor {} {
    global top_entity

    # goto project closed state (in work dir!)
    global xil_state_project_closed
    switch_state $xil_state_project_closed

    # goto project implemented
    global xil_state_project_implemented
    switch_state $xil_state_project_implemented

    # goto project closed state
    global xil_state_project_closed
    switch_state $xil_state_project_closed

    log_info "Starting FPGA Editor..."
    exec fpga_editor $top_entity.ncd $top_entity.pcf &
}

# start timing analyzer
proc start_timing_analyzer {} {
    global top_entity

    # goto project closed state (in work dir!)
    global xil_state_project_closed
    switch_state $xil_state_project_closed

    # goto project implemented
    global xil_state_project_implemented
    switch_state $xil_state_project_implemented

    # goto project closed state
    global xil_state_project_closed
    switch_state $xil_state_project_closed

    log_info "Starting Timing Analyzer..."
    exec timingan $top_entity.ncd $top_entity.pcf $top_entity.twx &
}

# start planahead
proc start_planahead {mode} {
    global xil_state_project_synced
    global xil_state_project_synthesized
    global xil_state_project_implemented

    global top_entity
    global device
    global package
    global speed
    global source_files

    #Note: simply executing process run "Analyze Timing / Floorplan Design (PlanAhead)" doesn't work

    # TODO; this could be autodetected
    if {$mode == 2} {
        switch_state $xil_state_project_synthesized
    } else {
        switch_state $xil_state_project_implemented
    }

    # create planahead startup script (same script as generated by ISE)
    set fs [open "pa.tcl" w]

    puts $fs "create_project -name $top_entity -dir \"pa\" -part $device$package$speed"
    puts $fs "set_property design_mode GateLvl \[get_property srcset \[current_run -impl\]\]"
    puts $fs "set_property edif_top_file \"$top_entity.ngc\" \[ get_property srcset \[ current_run \] \]"
    puts $fs "add_files -norecurse \".\""
    foreach source_file [array names source_files] {
        if {[string compare [string range $source_file end-2 end] "ucf"] == 0} {
            puts $fs "add_files [file join .. [lindex $source_files($source_file) 1]] -fileset \[get_property constrset \[current_run\]\]"
        }
    }
    puts $fs "open_netlist_design"
    if {$mode == 3} {
        puts $fs "read_xdl -file \"$top_entity.ncd\""
        puts $fs "if {\[catch {read_twx -name results_1 -file \"$top_entity.twx\"} eInfo\]} {"
        puts $fs "    puts \"WARNING: there was a problem importing \\\"$top_entity.twx\\\": \$eInfo\""
        puts $fs "}"
    }

    close $fs

    log_warning "PlanAhead may reformat all existing UCFs beyond repair, make sure all UCFs are backed up before saving!"

    log_info "Starting PlanAhead..."
    file delete -force "pa"
    exec planAhead -source pa.tcl &
}

# print the active logfile (if any), the logfile is expected to be found in the current working directory
# (which is work whenever one of the xilinx processes fail)
proc print_active_logfile {} {
    global active_logfile

    if {$active_logfile != ""} {
        log_info "Dumping $active_logfile:"

        set fs [open $active_logfile r]
        set file_data [read $fs]
        close $fs
        puts stderr $file_data
    }
}


# execute script commands
proc execute_commands {argc argv} {
    for {set i 0} {$i < $argc} {incr i} {
        switch [lindex $argv $i] {
            clean {
                cleanup
            }
            build {
                build 0
            }
            tig {
                build 1
            }
            synthesize {
                synthesize_only
            }
            log {
                print_latest_log
            }
            ise {
                start_ise
            }
            gui {
                start_ise
            }
            project {
                make_project
            }
            git_info {
                gen_git_info
            }
            insert {
                start_inserter
            }
            analyze {
                start_analyzer
            }
            impact {
                start_impact
            }
            edif {
                gen_edif
            }
            fpga_editor {
                start_fpga_editor
            }
            timing_analyzer {
                start_timing_analyzer
            }
            ta {
                start_timing_analyzer
            }
            planahead {
                start_planahead 3
            }
            pa {
                start_planahead 3
            }
            pa2 {
                start_planahead 2
            }
            pa3 {
                start_planahead 3
            }
            help {
                log_info "Usage: [lindex $argv $i] \[build|clean|ise|insert|analyze|impact|edif|fpga_editor|timing_analyzer|tig|planahead\]"
            }
            default {
                error "Unknown option '[lindex $argv $i]', use argument help for more info"
            }
        }
    }

    # back to start
    global xil_state_startup
    switch_state $xil_state_startup
}

# this is a wrapper for the xilinx build script.
# it makes sure that the project is correctly closed at all times, and that the
# script leaves in the same working directory as it started
proc run {argc argv} {
    global xil_state
    global protected
    global errors

    # record the start time
    set start_time [clock seconds]

    # keep track of current directory
    set curdir [pwd]

    # run commands with exception protection
    if {$protected == 0} {
        execute_commands $argc $argv
        # reprint all warnings, they are often hidden due to excessive logging from xilinx tools
        log_all_warnings
    } else {
        if [catch {execute_commands $argc $argv} error_message] {
            # print latest xilinx logfile
            catch {print_active_logfile}
            # this prevents ise file corruption
            terminate_fsm
            # reprint all warnings, they are often hidden due to excessive logging from xilinx tools
            log_all_warnings
            # print error message
            log_error $error_message
        }
    }

    set runtime [expr [clock seconds] - $start_time]
    if {$runtime < 300} {
        log_info "Done (took $runtime seconds)"
    } else {
        log_info "Done (took [expr $runtime / 60] minutes)"
    }

    return $errors
}

