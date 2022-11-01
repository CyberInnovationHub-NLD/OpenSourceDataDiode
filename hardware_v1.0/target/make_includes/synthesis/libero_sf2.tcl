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
set source_libs [list "work"]
set hdl_language VHDL
set timing_constraints_met false

# build settings, no not modify! use functions to alter them from the project file
set protected 1

# disable error catching, prints detailed errors but may currupt project file
proc disable_protection {} {
    global protected
    set protected 0
}

# add a source file to the list of source files
proc add_source {source_file {library "work"}} {
    global source_files
    global source_libs
    lappend source_files $source_file
    lappend source_files $library
}

set adv_options []
proc set_adv_option {option} {
    global adv_options
    lappend adv_options -adv_options $option
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

proc import_sources {} {
    global top_entity
    global source_files
    global source_libs
    set sdc_file ""
    set pdc_file ""
    
    # Create links to project sources. Use 'glob' to allow wild-card matching
    foreach {source_path library} $source_files {

        if {[lsearch $source_libs $library] == -1} {
            log_info "Create new library : $library"
            lappend source_libs $library
            add_library -library $library
        }
    
        foreach source_file [glob "$source_path"] {
            if {([file extension $source_file] == ".vhd") || ([file extension $source_file] == ".v")} {
                log_info "Add HDL source $source_file"
                create_links -hdl_source $source_file
                add_file_to_library -library $library -file $source_file
            } elseif {[file extension $source_file] == ".cxf"} {
                log_info "Add component $source_file"
                import_files -cxf $source_file
            } elseif {[file extension $source_file] == ".sdc"} {
                log_info "Add timing constraints file $source_file"
                create_links -sdc $source_file
                set sdc_file $source_file
            } elseif {[file extension $source_file] == ".pdc"} {
                log_info "Add io constraints file $source_file"
                create_links -io_pdc $source_file
                set pdc_file $source_file
            }
        }
    }
    
    #organize tool files for synthesis and compile stages
    set_root $top_entity
    set top_module [format "%s::work" $top_entity]
    organize_tool_files -tool {SYNTHESIZE} -file $sdc_file -module $top_module -input_type {constraint} 
    organize_tool_files -tool {COMPILE}    -file $sdc_file -file $pdc_file -module $top_module -input_type {constraint}     
}

proc create_project {} {
    global work_dir
    global top_entity
    global family
    global die
    global package
    global speed
    global source_files
    global adv_options
	global hdl_language
    
    catch {
        close_project
    }
    if {[file exists $work_dir]} {
        file delete -force $work_dir
    }

    log_info "Creating project $top_entity.prjx"
    new_project -name $top_entity -location $work_dir -family $family -die $die -package $package -speed $speed -hdl $hdl_language
    eval [concat set_device $adv_options]
}

proc check_timing_results {} {
	global timing_constraints_met
    global work_dir
    global top_entity
	
	set fn $work_dir/designer/$top_entity/${top_entity}_has_violations
	if {[file exists $fn]} {
		set f [open $fn]
		set verdict [gets $f]
		close $f
		set timing_constraints_met [string match -nocase "met" $verdict];
	}
}


################################################################################
# Project build steps.
###############################################################################
proc build_project {} {
	global target_name
	global work_dir
	global top_entity
	global timing_constraints_met
	
	set timing_constraints_met false;
	
	# Run synthesis/compile.
	log_info "Run tool: SYNTHESIS & COMPILE" 
	#run_synthesis
	if {[catch {run_tool -name {COMPILE}} err] } {
		log_fatal "Synthesis/compile failed!"
	}
	log_info "Compile successful!"
	
	# run place and route 'high effort' mode to meet timing constraints.
	log_info "Run tool: PLACEROUTE" 
	configure_tool -name {PLACEROUTE} -params {EFFORT_LEVEL:true} -params {INCRPLACEANDROUTE:false} -params {PDPR:false} -params {TDPR:true} 
	if {[catch {run_tool -name {PLACEROUTE}} err] } {
		log_fatal "Place & route failed!"
	}
	log_info "Place and Route finished successfully"
	
	log_info "Run tool: VERIFYTIMING" 
	run_tool -name {VERIFYTIMING} 
	log_info " Timing verification Finished"
	check_timing_results 
	if $timing_constraints_met {
		log_info "All timing constraints met!"
	} else {
		log_warning "Timing constraints NOT met!"
	}	
	
	# Generate and bit-file and export STAPL file
	log_info "Run tool: GENERATEPROGRAMMINGFILE" 
	if {[catch {run_tool -name {GENERATEPROGRAMMINGFILE}} err] } {
		log_fatal "Failed to generate programming file!"
	}
}

proc find_envm_image_data_size {size_info_file} {
	set size 128000
	
	if {[file exist size_info_file] == 1} {
		puts "ERROR: file not found $size_info_file"
		return $size
	}
	set f [open $size_info_file "r"]
	# Expect info file to be a size dump using 'arm-none-eabi-size --target=ihex <envm_img.hex>'

	# Skip first line: " text    data     bss     dec     hex filename"
	set result [gets $f line]		
	puts $line

	# second line, get data section size
	if {$result > 0} {set result [gets $f line]}
	puts $line
	if {$result > 0} {
		set sizes [regexp -inline -all -- {\S+} $line]
		set size [lindex $sizes 1]
	}

	close $f
	
	if {$size == 0} {
		# default size is MAX_ENVM size
		return 128000
	}
	return $size
}

proc create_envm_cfg {hex_image size} {
    set f [open "envm.cfg" "w"]
    puts -nonewline $f "nvm_set_data_storage_client -client_name {mss_data} -number_of_words $size -word_size 8 -use_for_simulation {0} "
    puts -nonewline $f "-content_type {MEMORY_FILE} -memory_file_format {INTELHEX} -memory_file {$hex_image} "
    puts $f "-base_address 0 -reprogram 1 -use_as_rom 0"
	close $f
}

proc export_bitstream {target_name} {

	global work_dir
	global top_entity
	global timing_constraints_met
    global envm_hex_image
    
    set target_file $target_name
    
    # Check if ENVM memory contents must be updated as well...    
    if {![info exists envm_hex_image]} {
        set target_file ${target_name}_no_envm
		log_info "Export fabric programming file ${target_file}.stp"	
		set components {FABRIC}
	} else {
		if {[file exists $envm_hex_image]} {
			set components {FABRIC ENVM}    
			log_info "Export combined FABRIC and ENVM programming file ${target_name}.stp"		
			create_envm_cfg $envm_hex_image [find_envm_image_data_size "${envm_hex_image}.size"]
			import_component_data -module $top_entity -fddr {} -mddr {} -serdes0 {} -serdes1 {} -serdes2 {} -serdes3 {} -envm_cfg {envm.cfg}
			log_info "ENVM hex image loaded: $envm_hex_image" 
		} else {
            set target_file ${target_name}_no_envm
			log_warning "ENVM hex image not found: $envm_hex_image, skipping...." 
            log_info "Export fabric programming file ${target_file}.stp"    
			set components {FABRIC}
		}
	}
	
	export_bitstream_file \
			 -file_name $target_file \
			 -format {STP} \
			 -limit_SVF_file_size 0 \
			 -limit_SVF_file_by_max_filesize_or_vectors {SIZE} \
			 -svf_max_filesize {1024} \
			 -svf_max_vectors {1000} \
			 -master_file 0 \
			 -master_file_components {} \
			 -encrypted_uek1_file 0 \
			 -encrypted_uek1_file_components {} \
			 -encrypted_uek2_file 0 \
			 -encrypted_uek2_file_components {} \
			 -trusted_facility_file 1 \
			 -trusted_facility_file_components $components \
			 -add_golden_image 0 \
			 -golden_image_address {} \
			 -golden_image_design_version {} \
			 -add_update_image 0 \
			 -update_image_address {} \
			 -update_image_design_version {} \
			 -serialization_stapl_type {SINGLE} \
			 -serialization_target_solution {FLASHPRO_3_4_5} 
	 
	# Copy image to target folder
	set image_file "$work_dir/designer/$top_entity/export/${target_name}.stp"
	if {[file exists $image_file]} {
		if $timing_constraints_met {
			file copy -force $image_file .
			log_info "Build Successfull. Generated output file ${target_name}.stp"
		} else {
			file copy -force $image_file ./${target_name}_with_issues.stp
			log_error "Build finished with timing violations. Generated output file ${target_name}_with_issues.stp"
		}
	} else {
		log_error "Build FAILED. No bitfile generated."
	}
}
