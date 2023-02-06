

pub fn print_report_without_header(top_changed_files: &Vec<(String, usize)>) {
    top_changed_files.iter()
    .for_each(|(file,num_changes)|{
        println!("{}\t{}",file, num_changes);
    });
}

pub fn print_report(top_changed_files: &Vec<(String, usize)>) {
    println!("{}", format!("{:80}", "-").replace(" ", "-"));
    println!("File\t\tNumber of changes");
    println!("{}", format!("{:80}", "-").replace(" ", "-"));
    print_report_without_header(top_changed_files);
}