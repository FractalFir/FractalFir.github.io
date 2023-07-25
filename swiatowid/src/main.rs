use clap::*;
use std::path::{Path,PathBuf};
use std::io::Read;
#[derive(Debug,Parser)]
struct Args{
    path:PathBuf,
}
#[derive(Debug)]
struct Metadata{
    title:String,
    id:String,
}
impl Metadata{
   fn from_string(string:&str)->Self{
        let mut res = Self{title:"Oh no! I forgot to put the title in.".to_string(),id:"".to_string()};
        for line in string.lines(){
            // There are no characters that are not whitespace(thus all are whitespace), skip.
            if !line.chars().any(|c|!c.is_whitespace()){
                continue;
            }
            let (variable,value) = {
                let mut tmp = line.split('=');
                let variable = tmp.next().expect("Syntax error! Line did not set a variable!");
                let value = tmp.next().expect("Syntax error! Variable set to nothing!");
                let variable = variable.trim();
                (variable,value)
            };
            match variable{
                "title"=>res.title = value.split('"').nth(1).expect("Title must be contained between 2 quotation marks!").to_string(),
                "id"=>res.id = value.split('"').nth(1).expect("ID must be contained between 2 quotation marks!").to_string(),
                _=>panic!("unknown variable \"{var}\"!",var = variable.escape_debug().to_string())
            }
            //println!("line:{line}");
        }
        if res.id.is_empty(){
            panic!("No unique article id!");
        }
        res
   }
}
#[derive(Debug)]
enum MarkdownElement{
    Raw(String),
}
#[derive(Debug)]
struct Markdown{
    elements:Box<[MarkdownElement]>,
}
fn header_depth(header:&str)->usize{
    for (char_offset,c) in header.chars().enumerate(){
        if c != '#'{
            return char_offset;
        }
    }
    header.len()
}
#[test]
fn header_detection(){
    assert_eq!(header_depth(""),0);
    assert_eq!(header_depth("#"),1);
    assert_eq!(header_depth("##"),2);
    assert_eq!(header_depth("###"),3);
    assert_eq!(header_depth("# #"),1);
    assert_eq!(header_depth("#A#"),1);
    assert_eq!(header_depth("A#"),0);
}
impl Markdown{
    fn from_string(string:&str)->Self{
        let mut elements = Vec::new();
        //let paragraph_text = String::new();
        let last_start = 0;
        let mut iter = string.chars().enumerate();
        while let Some((index,c)) = iter.next(){
            
        }
        elements.push(MarkdownElements::Raw(string[last_start..].to_owned()));
        Self{elements:elements.into()}
    }
    fn join(&mut self,other:Self){
        todo!();
    }
}
struct Article{
    metadata:Metadata,
    markdown:Markdown,
}
impl Article{
    fn from_file(mut file:impl Read)->std::io::Result<Self>{
        let mut content = String::with_capacity(0x1000);
        file.read_to_string(&mut content)?;
        
        let mut metadata_start:Vec<_> = (&content).match_indices("<metadata>").map(|(i,_)|i + "<metadata>".len()).collect();
        let mut metadata_end:Vec<_> = (&content).match_indices("</metadata>").map(|(i,_)|i).collect();
        let metadata:Vec<_> = metadata_start.iter().zip(metadata_end.iter()).map(|(start,end)|&content[(*start)..(*end)]).map(|string|Metadata::from_string(string)).collect();
        
        let mut markdown_start:Vec<_> = (&content).match_indices("<markdown>").map(|(i,_)|i + "<metadata>".len()).collect();
        let mut markdown_end:Vec<_> = (&content).match_indices("</markdown>").map(|(i,_)|i).collect();
        
        let mut markdown = markdown_start.iter().zip(markdown_end.iter()).map(|(start,end)|&content[(*start)..(*end)]).map(|string|Markdown::from_string(string));//.collect();
        let mut first_markdown = markdown.next().expect("Expected at least one markdown file!");
        for next_markdown in markdown{
            first_markdown.join(next_markdown);
        }
        let markdown = first_markdown;
        assert_eq!(metadata.len(),1,"Expected 1 and exactly one metadata field!");
        
        println!("metadata:{metadata:?},markdown:{markdown:?}");
        todo!("");
    }
}
fn collect_articles_from_dir(path:&Path)->std::io::Result<Vec<Article>>{
    let mut articles = Vec::new();
    for file in std::fs::read_dir(path)?{
        let file = file?;
        if file.file_type()?.is_file() && file.path().extension().is_some_and(|extension| extension == "fat_md"){
            let file = std::fs::File::open(file.path())?;
            let article = Article::from_file(file)?;
        }
        //println!("{}", file.unwrap().path().display());
    }
    Ok(articles)
}
fn main() {
    let args = Args::parse();
    let articles = collect_articles_from_dir(&args.path).unwrap();
    println!("Hello, world {args:?}!");
}
