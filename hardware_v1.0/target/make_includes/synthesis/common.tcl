set errors 0
set warnings 0
set warning_list []

proc log {line} {
    puts stderr "\[[clock format [clock seconds] -format %T]\] $line"
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

# Expand list of relative paths (may include wildcards) into list of absolute file paths
proc path_expand {path_list {offset "."}} {
    set expanded_paths {}
    # wildcard expansion
    foreach path $path_list {
        set expanded_paths [concat $expanded_paths [glob [file join $offset $path]]]
    }
    return $expanded_paths
}

# Apply offset and Return absolute path 
proc path_normalize {path {offset "."}} {
    return [file normalize [file join $offset $path]]
}
