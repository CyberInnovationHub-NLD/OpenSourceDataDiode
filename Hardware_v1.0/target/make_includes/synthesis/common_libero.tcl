set errors 0
set warnings 0
set warning_list []
set log_chan false

proc open_log {file_name} {
	# Redirect 'puts' output to log file.
	#rename puts ::tcl::orig::puts
	
	global log_chan
	set log_chan [open $file_name a]
	
	#proc puts args "
	#    uplevel \"::tcl::orig::puts \$args\"
	#	uplevel \"::tcl::orig::puts $log_chan \$args\"
	#    uplevel \"flush $log_chan\"
	#    uplevel \"flush stdout\"
	#	return
	#"
}

proc close_log {} {	
	global log_chan
	if {$log_chan != false} {
		close $log_chan
		set log_chan false
	}
	#rename puts {}
	#rename ::tcl::orig::puts puts
}

proc log {line} {
	global log_chan
	set msg "\[[clock format [clock seconds] -format %T]\] $line"
	puts $msg
	flush stdout
	if {$log_chan != false} {
		puts $log_chan $msg
		flush $log_chan
	}
}

proc log_info {line} {
    log "Info:    $line"
}

proc log_warning {line} {
    global warnings
    global warning_list
    log "Warning: $line"
    set warnings [expr $warnings + 1]
    lappend warning_list $line
}

proc log_all_warnings {} {
    global warning_list
    foreach warning $warning_list {
        log_warning $warning
    }
}

proc log_error {line} {
    global errors
    log "Error:   $line"
    set errors [expr $errors + 1]
}

proc log_fatal {line {exit_code 2}} {
    log "Fatal:   $line"
	error $exit_code
}
