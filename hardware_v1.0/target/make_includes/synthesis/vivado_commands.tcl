###############################################################################
## (C) COPYRIGHT 2009-2012 TECHNOLUTION BV, GOUDA NL
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
## Title      : Xilinx Vivado helper scripts
## Author     : Sijmen Woutersen <sijmen.woutersen@technolution.eu>
##            : Wei Man Chim <wei.man.chim@technolution.eu>
###############################################################################
## Description: 
###############################################################################

# protected mode
set protected 1

# debug
set print_state_changes 0

# active logfile (printed upon error)
set active_logfile ""

# ise version
set ise_version "0"

# script variables
set start_dir [pwd]
set work_dir "work"
set njobs 4
array set source_files []
array set xdc_files []
set settings []

# enter the work directory, create if needed
proc goto_work {} {
    global work_dir

    if [file exists $work_dir] {
        if [file isdirectory $work_dir] {
            cd $work_dir
        } else {
            error "A file named '$dir_name' exists"
        }
    } elseif {[string last "work" [pwd]] == [string length [pwd]]-4} {
        log_info "Already in work directory..."
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

proc add_source { source_file {file_type ""} } {
    global source_files
    set source_files($source_file) [list $source_file $file_type]
}

proc add_constraint { source_file {file_type ""} } {
    global xdc_files
    set xdc_files($source_file) [list $source_file $file_type]
}

# load a project, create if it doesn't exist
proc load_project {} {
    global top_entity
    global part_number
    global source_files
    global xdc_files
    global top_entity

    # check if the project is already there
    if [file exists $top_entity.xpr] {
        log_info "Loading project '$top_entity'..."
        open_project $top_entity.xpr
        set project_open 1
        set loaded_files [get_files]

    } else {
        log_info "Creating new project '$top_entity'..."
        create_project $top_entity -part $part_number
        set project_open 1
        set loaded_files [list]
        set_property target_language VHDL [current_project]
    }
    gen_git_info

    # add files
    log_info "Process source files"
    array set file_types []
    set source_paths { }
    foreach source_file [array names source_files] {
        foreach i [path_expand [lindex $source_files($source_file) 0] "../"] {
            set abs_path [path_normalize $i]

            if {[lsearch $loaded_files $abs_path] == -1} {
                log_info "Adding $i to project."
                lappend source_paths $i

                if {[lindex $source_files($source_file) 1] != ""} {
                    set file_types($i) [list $abs_path [lindex $source_files($source_file) 1]]
                }
            }
        }
    }
    if {[llength $source_paths] != 0} {
        add_files $source_paths
    }
    foreach fn [array names file_types] {
        log_info "Setting $fn to [lindex $file_types($fn) 1]"
        set_property file_type [lindex $file_types($fn) 1] [get_files [lindex $file_types($fn) 0]]
    }

    
    # add constraint files
    log_info "Process constraint files"
    array set file_types []
    set xdc_file_paths { }
    foreach xdc_file [array names xdc_files] {
        foreach i [path_expand [lindex $xdc_files($xdc_file) 0] "../"] {
            set abs_path [path_normalize $i]

            if {[lsearch $loaded_files $abs_path] == -1} {
                log_info "Adding $i to project."
                lappend xdc_file_paths $i

                if {[lindex $xdc_files($xdc_file) 1] != ""} {
                    set file_types($i) [list $abs_path [lindex $xdc_files($xdc_file) 1]]
                }
            }
        }
    }
    if {[llength $xdc_file_paths] != 0} {
        add_files -fileset constrs_1 -norecurse $xdc_file_paths
    }
    foreach fn [array names file_types] {
        log_info "Setting $fn to [lindex $file_types($fn) 1]"
        set_property file_type [lindex $file_types($fn) 1] [get_files [lindex $file_types($fn) 0]]
    }

    log_info "Setting toplevel to $top_entity"
    set_property top $top_entity [current_fileset]

    update_compile_order -fileset sources_1

    if {[info procs _do_user_settings] != ""} {
        log_info "Apply user project settings..."
        _do_user_settings
    }
    
}

# proc tcleanup {} {
proc cleanup {} {
    global work_dir
    global top_entity
    
    goto_start
    if [file exists $work_dir] {
        if [file isdirectory $work_dir] {
            # work dir 
            file delete -force $work_dir
        }

    } else {
        log_info "No work directory found to clean up."
    }
    # remove logs
    foreach f [glob -nocomplain vivado*.jou] {
        file delete -force $f
    }
    foreach f [glob -nocomplain vivado*.log] {
        file delete -force $f
    }
}

# generate git info
proc gen_git_info {} {
    global work_dir

    # check if we can run gitinfo
    if {[info procs {gitinfo_vivado}] != ""} {
        log_info "Generating Git Info..."
        gitinfo_vivado ".."
        add_source "$work_dir/tl_git_info/tl_git_info.vhd"
    }
}

proc proj_synth {} {
    global njobs
    
    # Mimic GUI behavior of automatically setting top and file compile order
    update_compile_order -fileset sources_1
    
    # check NEEDS_REFRESH flag
    if {[get_property NEEDS_REFRESH [get_runs synth_1]]} {
        log_info "Status of synth_1: [get_property STATUS [get_runs synth_1]], resetting run due to NEEDS_REFRESH flag."
        reset_run synth_1
    }
    
    # Launch Synthesis
    if {[get_property PROGRESS [get_runs synth_1]] != "100%"} {
        log_info "Launching run of synth_1."
        
        # check STATUS, only start from "Not started" status.
        if {[get_property STATUS [get_runs synth_1]] != "Not started"} {
            log_info "Status of synth_1: '[get_property STATUS [get_runs synth_1]]', resetting run to 'Not started'."
            reset_run synth_1
        }

        launch_runs synth_1 -jobs $njobs
        wait_on_run synth_1

        if {[get_property PROGRESS [get_runs synth_1]] != "100%"} {
            error "Synthesis failed"
        }
    }
    log_info "Status of synth_1: [get_property STATUS [get_runs synth_1]]"

}

proc proj_impl {} {
    global njobs
    
    # check NEEDS_REFRESH flag
    if {[get_property NEEDS_REFRESH [get_runs impl_1]]} {
        log_info "Status of impl_1: [get_property STATUS [get_runs impl_1]], resetting run due to NEEDS_REFRESH flag."
        reset_run impl_1
    }
    
    # Launch Implementation
    if {[get_property PROGRESS [get_runs impl_1]] != "100%"} {
        log_info "Launching run of impl_1."

        launch_runs impl_1 -jobs $njobs
        wait_on_run impl_1

        if {[get_property PROGRESS [get_runs impl_1]] != "100%"} {
            error "Implementation failed"
        }
    } 
    log_info "Status of impl_1: [get_property STATUS [get_runs impl_1]]"
}

proc proj_bitstream {} {
    global njobs
    
    if {[get_property NEEDS_REFRESH [get_runs impl_1]]} {
        log_warning " NEEDS_REFRESH flag is set, status of impl_1: [get_property STATUS [get_runs impl_1]], continueing with possibly out dated impl_1."
    }
    
    # Launch Implementation till write_bitstream
    if {[get_property STATUS [get_runs impl_1]] != "write_bitstream Complete!"} {
        launch_runs impl_1 -jobs $njobs -to_step write_bitstream
        wait_on_run impl_1
        
        if {[get_property PROGRESS [get_runs impl_1]] != "100%"} {
            error "Bitstream generation failed"
        }
    } else {
        if {[get_property NEEDS_REFRESH [get_runs impl_1]] != 0} {
            error "impl_1 NEEDS_REFRESH flag is set!"
        }
    }
    log_info "Status of impl_1: [get_property STATUS [get_runs impl_1]]"
}

# make a temporary tcl batch file
proc write_batch {batch_string batchfilename} {
    set batchfile [open $batchfilename "w"]
    puts $batchfile $batch_string
    close $batchfile
}

# start new instance gui with tcl commands
proc start_gui_args {batch_string} {
    set batchfilename "_batchwork.tcl"
    
    goto_work
    write_batch $batch_string $batchfilename
    exec vivado -mode gui -source $batchfilename &
}

proc debug_reload {} {
    set curr_dir [pwd]
    goto_start
    source "../scripts/vivado_slim.tcl"
    cd $curr_dir
}

####################################################################################################
####################################################################################################

# execute script commands
proc execute_commands {argc argv} {
    global top_entity
    for {set i 0} {$i < $argc} {incr i} {
        switch [lindex $argv $i] {
            clean {
                cleanup
            }
            gui {
                goto_work
                load_project
                start_gui_args "open_project ${top_entity}.xpr;"
            }
            project {
                goto_work
                load_project
            }
            synth {
                goto_work
                load_project
                proj_synth
            }
            impl {
                goto_work
                load_project
                proj_impl
            }
            build {
                goto_work
                load_project
                proj_synth
                proj_impl
                proj_bitstream
                if {[info procs _do_post_build_user_actions] != ""} {
                    log_info "Execute post-build user actions..."
                    _do_post_build_user_actions
                }
            }
            insert {
                goto_work
                load_project
                proj_synth
                # open_run synth_1 -name netlist_1
                # start_gui
                start_gui_args "open_project ${top_entity}.xpr; open_run synth_1 -name netlist_1;"
            }
            help {
                log_info "Usage: [lindex $argv $i] \[build|clean|ise|insert|analyze|impact|edif|fpga_editor|timing_analyzer|tig|planahead\]"
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
            # terminate_fsm
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
