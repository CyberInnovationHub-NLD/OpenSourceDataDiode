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
## Title      : Libero Build Command handler
## Author     : Sijmen Woutersen <sijmen.woutersen@technolution.eu>
###############################################################################
## Description: This scripts contains all supported xilinx toolchain commands
###############################################################################
set source_files []

# build settings, no not modify! use functions to alter them from the project file
set protected 1

# disable error catching, prints detailed errors but may currupt project file
proc disable_protection {} {
    global protected
    set protected 0
}

# add a source file to the list of source files
proc add_source {source_file} {
    global source_files
    lappend source_files $source_file
}

# cleanup the build environment
proc cleanup {} {
    global work_dir

    log_info "Performing cleanup..."
    file delete -force "$work_dir"
    file delete -force "designer_log"
    file delete -force "synth.log"
    file delete -force "\"n"
}

proc _open_project {} {
    global work_dir
    global top_entity
    global family
    global die
    global package
    global source_files

    if [file exists $work_dir] {
        log_info "Opening project $top_entity.prjx"
        open_project $work_dir/$top_entity.prjx
    } else {
        log_info "Creating project $top_entity.prjx"
        new_project -name $top_entity -location $work_dir -family $family -die $die -package $package -hdl VHDL
        foreach source_file $source_files {
            if {[file extension $source_file] == ".vhd"} {
                log_info "Add HDL source $source_file"
                create_links -hdl_source $source_file
            } elseif {[file extension $source_file] == ".vhdl"} {
                log_info "Add HDL source $source_file"
                create_links -hdl_source $source_file
            } elseif {[file extension $source_file] == ".sdc"} {
                log_info "Add SDC source $source_file"
                create_links -sdc $source_file
            } elseif {[file extension $source_file] == ".pdc"} {
                log_info "Add PDC source $source_file"
                create_links -pdc $source_file
            } else {
                log_info "Unknown file extension $source_file"
            }
        }
        set_root $top_entity

        save_project
    }
}

proc _close_project {} {
    close_project
}

proc gui {} {
    global work_dir
    global top_entity

    if {![file exists $work_dir]} {
        _open_project
        _close_project
    }
    log_info "Starting GUI in the background..."

    exec libero $work_dir/$top_entity.prjx &
}

proc _synthesize {} {
    set active_logfile "synth.log"
    run_synthesis -logfile synth.log
    set active_logfile ""
}

# build & analyze the design
proc synthesize {} {
    global active_logfile
    _open_project
    _synthesize
    save_project
    _close_project
}

# build & analyze the design
proc build {} {
    global active_logfile
    _open_project
    _synthesize
    save_project
    set active_logfile "designer.log"
    run_designer -logfile designer.log -adb new -compile TRUE -layout TRUE -export_ba TRUE
    set active_logfile ""
    save_project
    _close_project
}

# print the active logfile (if any), the logfile is expected to be found in the current working directory
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
            synthesize {
                synthesize
            }
            build {
                build
            }
            gui {
                gui
            }
            default {
                error "Unknown option '[lindex $argv $i]', use argument help for more info"
            }
        }
    }
}

# this is a wrapper for the xilinx build script.
# it makes sure that the project is correctly closed at all times, and that the
# script leaves in the same working directory as it started
proc run {argc argv} {
    global protected
    global errors

    # record the start time
    set start_time [clock seconds]

    # keep track of current directory
    set curdir [pwd]

    # run commands with exception protection
    if {$protected == 0} {
        execute_commands $argc $argv
    } else {
        if [catch {execute_commands $argc $argv} error_message] {
            # print latest logfile
            catch {print_active_logfile}
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

