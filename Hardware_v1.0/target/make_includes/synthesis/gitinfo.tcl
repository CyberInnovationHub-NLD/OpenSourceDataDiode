###############################################################################
## (C) COPYRIGHT 2010-2017 TECHNOLUTION BV, GOUDA NL
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
## Title      : GitInfo
## Author     : Sijmen Woutersen <sijmen.woutersen@technolution.nl>
###############################################################################
## Description: Generate a VHDL package containing git information of all
##              project sources and build date/time
###############################################################################


###############################################################################
## Create git info structure of all files in the archive
## The $dir argument specifies the dir all source files are relative to
###############################################################################
set hash 0000000000000000000000000000000000000000
set dirty 0

proc gitinfo_gather_info {dir {force_dirty 0}} {
    global hash
    global dirty

    set hash [exec git rev-parse HEAD]

    set dirty $force_dirty
    set local_modifications 0
    foreach line [split [exec git status -s] "\n"] {
        set local_modifications [expr $local_modifications + 1]
        set dirty 1
    }

    set count [llength [split [exec git log --oneline] "\n"]]

    if {$dirty == 0} {
        # archive is clean -> return date & revision of last change
        set commit_time [exec git log -n 1 --pretty=format:%at]

        return [list \
                        $hash \
                        [clock format $commit_time -format {%Y%m%d}] \
                        [clock format $commit_time -format {%H%M%S}] \
                        "00000000" \
                        0 \
                        $count \
               ]
    } else {
        # archive is dirty -> return current date & and revision(s) of working copy
        set start_time [clock seconds]

        return [list \
                        $hash \
                        [clock format $start_time -format {%Y%m%d}] \
                        [clock format $start_time -format {%H%M%S}] \
                        [format %08X [clock format $start_time -format {%s}]] \
                        1 \
                        $count \
               ]
    }
}

###############################################################################
## Generate git info VHDL file (entity)
###############################################################################
proc gitinfo_generate {entity_name data} {
    set fs [open "$entity_name.vhd" w]
    puts $fs "-- This file is auto-generated, do not edit!"
    puts $fs ""
    puts $fs "library ieee;"
    puts $fs "use ieee.std_logic_1164.all;"
    puts $fs "use ieee.numeric_std.all;"
    puts $fs ""
    puts $fs "entity $entity_name is"
    puts $fs "    port ("
    puts $fs "        -- use msbs if a smaller hash is preferred (e.g.: most significant 32-bits)"
    puts $fs "        hash                  : out std_logic_vector(159 downto 0);"
    puts $fs "        -- nr of commits up uptill this one, only ever use this for release versioning if"
    puts $fs "        -- use always release from the same branch"
    puts $fs "        commit_count          : out unsigned(31 downto 0);"
    puts $fs "        build_date            : out unsigned(31 downto 0);"
    puts $fs "        build_time            : out unsigned(23 downto 0);"
    puts $fs "        build_unix_time       : out unsigned(31 downto 0);"
    puts $fs "        is_dirty              : out std_logic"
    puts $fs "    );"
    puts $fs "end entity;"
    puts $fs ""
    puts $fs "architecture struct of $entity_name is"
    puts $fs "begin"
    puts $fs "    hash                  <= x\"[lindex $data 0]\";"
    puts $fs "    commit_count          <= to_unsigned([lindex $data 5], 32);"
    puts $fs "    build_date            <= x\"[lindex $data 1]\";"
    puts $fs "    build_time            <= x\"[lindex $data 2]\";"
    puts $fs "    build_unix_time       <= x\"[lindex $data 3]\";"
    puts $fs "    is_dirty              <= '[lindex $data 4]';"
    puts $fs "end architecture;"
    close $fs
}

###############################################################################
## Store difference between max and min revision
###############################################################################
proc gitinfo_generate_diff {dir dir_name data} {
    set max_revision [lindex $data 2]

    set name "tl_git_info_[lindex $data 3]_[lindex $data 4]"

    log_info "Storing git_info.vhd and any differences with archive in $name."

    set diff_dir_name $name

    if [file exists $diff_dir_name] {
        error "Please wait a second, then try again (timebased directory already exists)"
    }
    
    file mkdir $diff_dir_name
    
    #file copy -force [file join $dir_name tl_git_info.vhd] $diff_dir_name

    # generate diff
    exec git diff [lindex $data 0] > [file join $diff_dir_name [lindex $data 3].diff]
}

###############################################################################
## Xilinx ISE specific git info code, creates a new ngc file containing the
## git info entity
###############################################################################
proc gitinfo_xilinx {dir {force_dirty 0}} {
    global family
    global device
    global package
    global speed

    set dir_name  "tl_git_info"

    # run git info
    set gitinfo_data [gitinfo_gather_info $dir $force_dirty]

    if [file exists $dir_name] {
        file delete -force $dir_name
    }

    file mkdir $dir_name
    cd $dir_name

    # create entity file
    gitinfo_generate "tl_git_info" $gitinfo_data

    # create netlist
    project new "tl_git_info"
    project set family $family
    project set device $device
    project set package $package
    project set speed $speed
    xfile add "tl_git_info.vhd"
    project set "Add I/O Buffers" "False" -process "Synthesize - XST"
    process run "Synthesize - XST"
    project close

    file copy -force "tl_git_info.ngc" ".."
    file delete -force "../tl_git_info.edf"

    cd ".."

    # create diffs (only if dirty)
    if {[lindex $gitinfo_data 6] != 0} {
        catch {gitinfo_generate_diff $dir $dir_name $gitinfo_data}
    }

    return "tl_git_info.ngc"
}

###############################################################################
## Xilinx Vivado specific git info code, creates a new ngc file containing the
## git info entity
###############################################################################
proc gitinfo_vivado {dir} {
    global family
    global device
    global package
    global speed

    set dir_name  "tl_git_info"

    # run git info
    set gitinfo_data [gitinfo_gather_info $dir]

    if [file exists $dir_name] {
        file delete -force $dir_name
    }

    file mkdir $dir_name
    cd $dir_name

    # create entity file
    gitinfo_generate "tl_git_info" $gitinfo_data

    # # create netlist
    # project new "tl_git_info"
    # project set family $family
    # project set device $device
    # project set package $package
    # project set speed $speed
    # xfile add "tl_git_info.vhd"
    # project set "Add I/O Buffers" "False" -process "Synthesize - XST"
    # process run "Synthesize - XST"
    # project close

    #file copy -force "tl_git_info.ngc" ".."
    # file delete -force "../tl_git_info.edf"
    file copy -force "tl_git_info.vhd" ".."



    # create diffs (only if dirty)
    if {[lindex $gitinfo_data 6] != 0} {
        catch {gitinfo_generate_diff $dir $dir_name $gitinfo_data}
    }
    cd ..
    return "tl_git_info.vhd"
}

###############################################################################
## Altera specific git info code, creates a VHDL file which is added to the
## project
###############################################################################
proc gitinfo_altera {dir project} {
    set dir_name  "tl_git_info"

    post_message "Running git info for $project"


    # run git info
    set gitinfo_data [gitinfo_gather_info $dir]

    if [file exists $dir_name] {
        file delete -force $dir_name
    }

    file mkdir $dir_name
    cd $dir_name

    # create entity file
    gitinfo_generate "tl_git_info" $gitinfo_data

    cd ".."

    project_open $project
    set_global_assignment -name VHDL_FILE [file join $dir_name "tl_git_info.vhd"]
    project_close

    # create diffs (only if dirty)
    if {[lindex $gitinfo_data 6] != 0} {
        if [catch {gitinfo_generate_diff $dir $dir_name $gitinfo_data} error_message] {
            log_warning "Could not generate git diffs: $error_message"
        }
    }

}

###############################################################################
## Standalone git info VHDL code generation
###############################################################################
proc gitinfo_standalone {dir} {
    set dir_name  "tl_git_info"

    #post_message "Running standalone git info generation"


    # run git info
    set gitinfo_data [gitinfo_gather_info $dir]

    if [file exists $dir_name] {
        file delete -force $dir_name
    }

    file mkdir $dir_name
    cd $dir_name

    # create entity file
    gitinfo_generate "tl_git_info" $gitinfo_data

    cd ".."


    # create diffs (only if dirty)
    if {[lindex $gitinfo_data 6] != 0} {
        if [catch {gitinfo_generate_diff $dir $dir_name $gitinfo_data} error_message] {
            #log_warning "Could not generate git diffs: $error_message"
        }
    }

}

###############################################################################
## Script entry point
###############################################################################
if {[info procs {xfile}] != ""} {
    # xilinx: do nothing for now, the xilinx build scripts will call
    # gitinfo_xilinx when needed
    log_info "Xilinx ISE environment detected"
} elseif {[info exists quartus] == 1} {
    # altera: run directly, this script is (should be) started from the
    # altera build-chain as standalone script
    # include common code (logging), altera specific
    source [file join [file dirname [info script]] "common_altera.tcl"]
    # make sure we're called post-compile
    if [string match "compile" [lindex $quartus(args) 0]] {
        gitinfo_altera "." [lindex $quartus(args) 1]
    } else {
        log_warning "Git info called at wrong build stage, doing nothing!"
    }
} elseif {[string last "vivado" [info nameofexecutable]] > 0} {
    # error "Xilinx Vivado environment detected TODO implement this"
    log_info "Git INFO: Xilinx Vivado environment detected"
} else {
    # genarate standalone git info module
    log_info "Running standalone git info generation"
    gitinfo_standalone "."
}
