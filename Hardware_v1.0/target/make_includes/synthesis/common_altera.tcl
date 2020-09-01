set errors 0
set warnings 0

proc log {line} {
    post_message $line
}

proc log_info {line} {
    post_message -type info $line
}

proc log_warning {line} {
    global warnings
    post_message -type warning $line
    set warnings [expr $warnings + 1]
}

proc log_error {line} {
    global errors
    post_message -type error $line
    set errors [expr $errors + 1]
}

