use {
    clap::Parser, std::{path::{PathBuf, Path}, error::Error},
    tokio::{fs, io::AsyncWriteExt}, regex::Regex
};



#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    #[arg(short, long, default_value_t = String::from("main.c"))]
    file: String,
}



async fn remove_headers(src: &str, parent_path: &Path) -> Result<String, Box<dyn Error>> {
    let include_reg = Regex::new(r#"#include ["<]([A-Za-z0-9_/.]*)[">]"#)?;

    let mut result = src.to_owned();
    let mut tmp = String::new();

    loop {
        tmp.clone_from(&result);

        let mut replaced_any = false;

        for (full, [rel_path]) in include_reg.captures_iter(&result).map(|c| c.extract()) {
            let path = parent_path.join(rel_path);
            
            let Ok(header_content) = fs::read_to_string(path).await
            else {
                continue;
            };
            
            replaced_any = true;

            tmp = tmp.replace(full, &header_content);
        }

        if !replaced_any {
            break;
        }

        result.clone_from(&tmp);
    }

    Ok(result)
}



#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let file_name = PathBuf::from(CliArgs::parse().file);

    let mut main_contents = fs::read_to_string(&file_name).await?;
    let mut new_contents = String::new();

    let include_reg
        = Regex::new(r#"#include ["<](lib/cstlite/[A-Za-z0-9_/.]*)[">]"#)?;

    loop {
        new_contents.clone_from(&main_contents);

        let mut captured_any = false;

        for (full, [path]) in include_reg.captures_iter(&main_contents).map(|c| c.extract()) {
            captured_any = true;

            let mut include_content = fs::read_to_string(path).await?;

            let mut parent_path = PathBuf::from(path);
            parent_path.pop();

            include_content.clone_from(
                &remove_headers(&include_content, &parent_path).await?
            );

            new_contents = new_contents.replace(full, &include_content);
        }

        if !captured_any {
            break;
        }

        main_contents.clone_from(&new_contents);
    }

    let mut file = fs::File::create(file_name).await?;
    file.write_all(new_contents.as_bytes()).await?;

    Ok(())
}