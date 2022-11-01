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
## Title      : Xilinx Build FSM
## Author     : Sijmen Woutersen <sijmen.woutersen@technolution.eu>
###############################################################################
## Description: This scripts contains an FSM which is in sync with the state
##              of a project during the build. It exports a 'switch_state'
##              function which can be used to force the project into a certain
##              state, e.g.: opened, synthesized, implented etc.
###############################################################################

# states identifiers of the build script, note, these are sequential and always executed sequantially
# in either direction
set xil_state_startup               0
set xil_state_project_closed        1
set xil_state_project_open          2
set xil_state_project_synced        3
set xil_state_project_synthesized   4
set xil_state_project_implemented   5
set xil_state_project_bitfile       6

set xil_state_dead                  9

# current state of the build script
set xil_state $xil_state_startup

# debug
set print_state_changes 0

# active logfile (printed upon error)
set active_logfile ""

# ise version
set ise_version "0"

# script variables
set start_dir [pwd]
set work_dir "work"
array set source_files []
set settings []

# add a source file to the list of source files
proc add_source {source_file {library ""}} {
    global source_files
    global libraries
    set source_files([file tail $source_file]) [list $library $source_file]
    if {$library != ""} {
        set libraries($library) $library
    }
}

# add a setting to the list of settings
proc set_setting {process setting value} {
    global settings
    lappend settings [list $process $setting $value]
}

# set version to build with, used for verification only
proc ise_version {version} {
    global ise_version
    set ise_version $version
}


# enter the work directory, create if needed
proc goto_work {} {
    global work_dir

    if [file exists $work_dir] {
        if [file isdirectory $work_dir] {
            cd $work_dir
        } else {
            error "A file named '$dir_name' exists"
        }
    } else {
        log_info "Creating work directory..."
        file mkdir $work_dir
        cd $work_dir
    }
}

# goto start directory
proc goto_start {} {
    global start_dir
    cd $start_dir
}

# obtain & check current version
proc check_version {} {
    global ise_version
    global ise_version_major
    global ise_version_minor

    # make sure the correct ISE version is used
    catch {exec map > map_version}
    set fp [open "map_version" r]
    set data [read $fp]
    close $fp
    file delete -force "map_version"

    # break the "Release Major.Minor.Revision" line using regexp, make sure to remove all leading zeros
    regexp {Release ([0\.]*)([0-9]+)([0\.]*)([0-9]+)([0\.]*)([0-9]+)?} $data all ig1 major ig2 minor ig3 revision
    if [string compare $revision "" ] {
        set version_string "$major.$minor.$revision"
    } else {
        set version_string "$major.$minor"
    }
    log_info "Using ISE Version:    $version_string"

    set ise_version_major $major
    set ise_version_minor $minor

    # if an ise version is specified, verify it
    if [string compare $ise_version ""] {
        if [string compare $version_string $ise_version] {
            error "Project is being build using an incorrect ISE version (need $ise_version)"
        }
    } else {
        log_warning "ISE version not specified"
    }
}

# load a project, create if it doesn't exist
proc load_project {} {
    global family
    global device
    global package
    global speed
    global top_entity
    global ise_version_major

    if [expr $ise_version_major >= 12] {
        set ise_ext "xise"
    } else {
        set ise_ext "ise"
    }

    # check if the project is already there
    if [file exists $top_entity.$ise_ext] {
        log_info "Loading project '$top_entity'..."
        project open $top_entity.$ise_ext
        set project_open 1
    } else {
        log_info "Creating new project '$top_entity'..."
        project new $top_entity.$ise_ext
        project set "Preferred Language" "VHDL"
        set project_open 1

        log_info "Setting device family to $family..."
        project set family $family
        log_info "Setting device to $device..."
        project set device $device
        log_info "Setting device package to $package..."
        project set package $package
        log_info "Setting device speed-grade to $speed..."
        project set speed $speed
    }

    # verify the device info, it seems impossible to change these values later on (they jump back to their previous values)
    # also, an invalid option is silently discarded, so checking is extremely important!
    if [string compare [string tolower [project get "family"]] [string tolower $family]] {
        error "Device family changed or invalid, please perform a cleanup"
    }
    if [string compare [string tolower [project get "device"]] [string tolower $device]] {
        error "Device changed or invalid, please perform a cleanup"
    }
    if [string compare [string tolower [project get "package"]] [string tolower $package]] {
        error "Device package changed or invalid, please perform a cleanup"
    }
    if [string compare [string tolower [project get "speed"]] [string tolower $speed]] {
        error "Device speed-grade changed or invalid, please perform a cleanup"
    }
}

# close an open project
proc close_project {} {
    project close
}

# add a single source file to the open project
proc add_file_to_project {source_file {library ""}} {
    # we are in working directory now
    set source_file [file join ".." $source_file]

    if [file isfile $source_file] {
        if [file readable $source_file] {
            # check if file is present in archive
            if [catch {xfile get [file tail $source_file] name} error_message] {
                # error => not available => add it
                if {$library == ""} {
                    xfile add $source_file
                    log_info "Source $source_file added to project"
                } else {
                    xfile add $source_file -lib_vhdl $library
                    log_info "Source $source_file added to project (library $library)"
                }
            }
        } else {
            log_info "Warning: can not read from $source_file, not adding to project."
        }
    } else {
        log_info "Warning: $source_file does not exist, not adding to project."
    }
}

# set a single setting to the open project
proc set_setting_to_project {process setting_name value} {
    # note: tcl crashes on some occasions when setting a setting to the previous value, so set them only when
    #       it is changed. also; sometimes ISE simply ignores the set, so recheck if the change was done

    # try to obtain current value, this may fail for some settings (e.g.: Macro Search Path)
    set current_value ""
    catch {set current_value [project get $setting_name -process $process]}

    if [string compare [string tolower $current_value] [string tolower $value]] {
        # setting is different than current
        log_info "Changing '$setting_name' to '$value' for process '$process'";
        project set $setting_name $value -process $process

        # recheck, invalid settings may be silently ignored
        catch {set current_value [project get $setting_name -process $process]}

        if [string compare [string tolower $current_value] [string tolower $value]] {
            # not the same after setting
            log_warning "Setting $setting_name silently ignored by ISE"
        }
    }
}

# sync all settings and source files provided in the tcl file with the ise file
proc sync_project {} {
    global source_files
    global libraries
    global settings
    global errorInfo
    global family
    global device
    global package
    global speed
    global top_entity
    global top_architecture
    global work_dir

    # create libraries
    foreach library [array names libraries] {
        if [catch {lib_vhdl get $library name}] {
            log_info "Creating library $library"
            lib_vhdl new $library
        }
    }

    set regenerate_cores 0
    set coregens []
    # add all files in array but not in project
    foreach source_file [array names source_files] {
        if [catch {xfile get $source_file name}] {
            add_file_to_project [lindex $source_files($source_file) 1] [lindex $source_files($source_file) 0]
        }
        if {[string compare [string range $source_file end-2 end] "xco"] == 0} {
            # comes in pair
            set xise_file "[string range [lindex $source_files($source_file) 1] 0 end-3]xise"
            set source_files([file tail $xise_file]) [list [lindex $source_files($source_file) 0] $xise_file]

            # see if regeneration is needed
            set filename [file join ".." [lindex $source_files($source_file) 1]]
            set coregen_log [file join [file dirname $filename] "coregen.log"]
            if {[file exist $coregen_log] == 0 || [file mtime $coregen_log] < [file mtime $filename]} {
                log_info "Found coregen: $filename (out of date)"
                set regenerate_cores 1
            } else {
                log_info "Found coregen: $filename (up to date)"
            }

            lappend coregens $filename
        }
    }

    # (re)add cdc file if found
    if [file isfile [file join ".." $top_entity.cdc]] {
        if [catch {xfile get $top_entity.cdc name}] {
            log_info "Adding chipscope to project"
            add_file_to_project $top_entity.cdc
        } else {
            log_info "Building with chipscope"
        }
    } else {
        if ![catch {xfile get $top_entity.cdc name}] {
            log_info "Removing chipscope from project (cdc deleted)"
            xfile remove $top_entity.cdc
        }
    }

    # remove all files in project but not in array
    foreach source_file [search * -type file] {
        if {[file extension [object name $source_file]] != ".cdc"} {
            if [catch {set t $source_files([file tail [object name $source_file]])} msg] {
                log_info "Removing [file tail [object name $source_file]] from project..."
                xfile remove [file tail [object name $source_file]]
            }
        }
    }

    # update toplevel
    if [string compare [file tail [project get top]] [file tail $top_entity]] {
        log_info "Setting top-level instance"
        project set top $top_architecture $top_entity

        if [string compare [file tail [project get top]] [file tail $top_entity]] {
            # setting the top failed, something is terribly wrong
            log_info "'[project get top]' - '$top_entity'"
            error "Could not set top-level instance, likely caused by malformed VHDL"
        }
    }

    # synchronize settings
    foreach setting $settings {
        if [catch {set_setting_to_project [lindex $setting 0] [lindex $setting 1] [lindex $setting 2]} errorInfo] {
            log_warning "Could not set setting '[lindex $setting 1]' for process '[lindex $setting 0]', ignoring..."
        }
    }

    # regenerate coregen
    if {$regenerate_cores != 0} {
        log_info "(Re)generating coregen cores..."
        process run "Regenerate All Cores"

        # remove date
        foreach cg_file $coregens {
            exec sed -i "s/^# Date:.*/# Date: (removed)/g" $cg_file
        }
    }

    project save
}

# check status of given build phase, stop script if failed
proc check_errors {phase} {
    global status
    set status [ process get $phase status ]
    if { $status == "out_of_date" } {
        log_info "$phase is out of date, it may be run again automatically during the next step"
    } elseif { $status != "warnings" && $status != "up_to_date" && $status != "never_run" } {
        log_info "$phase FAILED: return value: $status"
        if { $phase == "Translate" } {
            log_info "If the translate logging does not show any errors, the problem might be caused by Chipscope, try running the inserter to see if all signals are connected"
        }
        error "$phase FAILED: return value: $status"
    }
}

# check status of given build phase, return 1 if successfully done
proc check_done {phase} {
    global status
    set status [ process get $phase status ]
    if { $status != "warnings" && $status != "up_to_date" } {
        return 0
    } else {
        return 1
    }
}

# synthesize a design
proc synthesize {} {
    global active_logfile
    global top_entity

    if {[check_done "Synthesize - XST"]} {
        log_info "Synthesis up-to-date -> skipping"
    } else {
        if {[info procs {pre_synthesis}] != ""} {
            log_info "Running pre_synthesis script."
            pre_synthesis "$top_entity"
        }

        log_info "Synthesizing..."
        set active_logfile "$top_entity.syr"
        process run "Synthesize - XST"
        check_errors "Synthesize - XST"
        set active_logfile ""

        if {[info procs {post_synthesis}] != ""} {
            log_info "Running post_synthesis script."
            post_synthesis "$top_entity.ngc"
        }
    }
}

proc is_cpld {} {
    if [regexp -line {Map} [project get_processes] result] {
        return 0
    } else {
        return 1
    }
}

# implement a design
proc implement {} {
    global active_logfile
    global top_entity
    set hook_called "0"

    if {[check_done "Translate"]} {
        log_info "Translate up-to-date -> skipping"
        log_warning "Git Info is not updated!"
    } else {
        # check if we can run gitinfo
        if {[info procs {gitinfo_xilinx}] != ""} {
            close_project
            log_info "Generating Git Info..."
            gitinfo_xilinx ".."
            load_project
        }

        if {[info procs {pre_implement}] != "" && $hook_called == "0"} {
            log_info "Running pre_implement script."
            pre_implement "$top_entity"
            set hook_called "1"
        }

        log_info "Translating design..."
        set active_logfile "$top_entity.bld"
        process run "Translate"
        check_errors "Translate"
        set active_logfile ""
    }

    if [is_cpld] {
        if {[check_done "Fit"]} {
            log_info "Fit up-to-date -> skipping"
        } else {
            if {[info procs {pre_implement}] != "" && $hook_called == "0"} {
                log_info "Running pre_implement script."
                pre_implement "$top_entity"
                set hook_called "1"
            }

            log_info "Fitting design..."
            set active_logfile "${top_entity}.rpt"
            process run "Fit"
            check_errors "Fit"
            set active_logfile ""
        }
    } else {

        if {[check_done "Map"]} {
            log_info "Map up-to-date -> skipping"
        } else {
            if {[info procs {pre_implement}] != "" && $hook_called == "0"} {
                log_info "Running pre_implement script."
                pre_implement "$top_entity"
                set hook_called "1"
            }

            log_info "Mapping design..."
            set active_logfile "${top_entity}_map.map"
            process run "Map"
            check_errors "Map"
            set active_logfile ""
        }

        if {[check_done "Place & Route"]} {
            log_info "Place & Route up-to-date -> skipping"
        } else {
            if {[info procs {pre_implement}] != "" && $hook_called == "0"} {
                log_info "Running pre_implement script."
                pre_implement "$top_entity"
                set hook_called "1"
            }

            log_info "Routing design..."
            set active_logfile "$top_entity.par"
            process run "Place & Route"
            check_errors "Place & Route"
            set active_logfile ""
        }
    }

    if {[info procs {post_implement}] != "" && $hook_called == "1"} {
        log_info "Running post_implement script."
        post_implement "$top_entity.ngd"
    }
}

# generate the bitfile
proc generate_bitfile {} {
    global active_logfile
    global top_entity

    if {[check_done "Generate Programming File"]} {
        log_info "Generate Programming File up-to-date -> skipping"
    } else {
        if {[info procs {pre_bitgen}] != ""} {
            log_info "Running pre_bitgen script."
            pre_bitgen "$top_entity"
        }

        log_info "Generating bitfile..."
        set active_logfile "$top_entity.bgn"
        process run "Generate Programming File"
        check_errors "Generate Programming File"
        set active_logfile ""

        if {[info procs {post_bitgen}] != ""} {
            log_info "Running post_bitgen script."
            post_bitgen "$top_entity.bit"
        }
    }
}

# switch the current state of the build script to $new_state
proc switch_state {new_state} {
    global xil_state
    global xil_state_startup
    global xil_state_project_closed
    global xil_state_project_open
    global xil_state_project_synced
    global xil_state_project_synthesized
    global xil_state_project_implemented
    global xil_state_project_bitfile
    global print_state_changes

    # debug
    if {$print_state_changes} {
        log_info "Starting state change from $xil_state -> $new_state"
    }

    # exectute the fsm until the desired state is reached
    # note; this fsm is sequential, each state can go forward or backward one state, no states can be skipped
    while {$xil_state != $new_state} {
        if {$xil_state == $xil_state_startup} {
            if {$new_state > $xil_state} {
                # forward: enter work directory
                goto_work
                check_version
                set xil_state $xil_state_project_closed
            } else {
                # backward
                error "Unimplemented toolchain path ($xil_state->$new_state)"
            }
        } elseif {$xil_state == $xil_state_project_closed} {
            if {$new_state > $xil_state} {
                # forward: open project
                load_project
                set xil_state $xil_state_project_open
            } else {
                # backward, return to starting directory
                goto_start
                set xil_state $xil_state_startup
            }
        } elseif {$xil_state == $xil_state_project_open} {
            if {$new_state > $xil_state} {
                # forward: synchronize the project settings
                sync_project
                set xil_state $xil_state_project_synced
            } else {
                # backward (close project)
                close_project
                set xil_state $xil_state_project_closed
            }
        } elseif {$xil_state == $xil_state_project_synced} {
            if {$new_state > $xil_state} {
                # forward: synthesize
                synthesize
                set xil_state $xil_state_project_synthesized
            } else {
                # backward (no need to do anything
                set xil_state $xil_state_project_open
            }
        } elseif {$xil_state == $xil_state_project_synthesized} {
            if {$new_state > $xil_state} {
                # forward: implement
                implement
                set xil_state $xil_state_project_implemented
            } else {
                # backward (no need to do anything
                set xil_state $xil_state_project_synced
            }
        } elseif {$xil_state == $xil_state_project_implemented} {
            if {$new_state > $xil_state} {
                generate_bitfile
                set xil_state $xil_state_project_bitfile
            } else {
                # backward (no need to do anything
                set xil_state $xil_state_project_synthesized
            }
        } elseif {$xil_state == $xil_state_project_bitfile} {
            if {$new_state > $xil_state} {
                # forward
                error "Unimplemented toolchain path ($xil_state->$new_state)"
            } else {
                # backward (no need to do anything
                set xil_state $xil_state_project_implemented
            }
        } else {
            error "Unknown toolchain state ($xil_state)"
        }

        # debug
        if {$print_state_changes} {
            log_info "Current state: $xil_state"
        }
    }
}


# force exit the fsm, must be used after an error to prevent project file corruption
# this should be as safe as possible, and never return errors
proc terminate_fsm {} {
    global start_dir
    global xil_state
    global xil_state_project_open

    if {$xil_state >= $xil_state_project_open} {
        project close
    }

    cd $start_dir

    # unknown state now, better restart
    set xil_state xil_state_dead
}
