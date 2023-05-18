use crate::TopComplexities;
use anyhow::Result;
use complexity_radar::ChangedFileCounts;

pub fn print_report_without_header(top_changed_files: &ChangedFileCounts) {
    top_changed_files.iter().for_each(|(file, num_changes)| {
        println!("{}\t{}", file, num_changes);
    });
}

pub fn print_top_complexities_report_without_header(top_complexities: &TopComplexities) {
    println!(
        "{}\t{}",
        top_complexities.code_filename, top_complexities.num_changes
    );
    top_complexities
        .function_complexities
        .iter()
        .for_each(|function_complexity| {
            println!(
                "\t{}\t{}",
                function_complexity.function, function_complexity.cognitive_complexity_value
            );
        })
}

pub fn print_heat_map_report(top_changed_files: &ChangedFileCounts) {
    println!("{}", format!("{:80}", "-").replace(" ", "-"));
    println!("File\t\tNumber of changes");
    println!("{}", format!("{:80}", "-").replace(" ", "-"));
    print_report_without_header(top_changed_files);
}

pub fn print_top_complexities_report(top_changed_files: &Vec<Result<TopComplexities>>) {
    println!("{}", format!("{:80}", "-").replace(" ", "-"));
    println!("File\t\tNumber of changes");
    println!("{}", format!("{:80}", "-").replace(" ", "-"));
    top_changed_files
        .iter()
        .flatten()
        //.filter_map(|top_complexities| top_complexities.ok().map(|top_complexity| top_complexity))
        .for_each(|top_complexities| {
            print_top_complexities_report_without_header(top_complexities);
        });
}
