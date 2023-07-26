use clap::*;
use std::path::{Path,PathBuf};
use std::io::Read;
use std::collections::HashMap;
#[derive(Debug,Parser)]
struct Args{
    path:PathBuf,
}
#[derive(Debug)]
struct Metadata{
    title:String,
    id:String,
    style:String,
    topic_id:String,
}
impl Metadata{
    fn id(&self)->&str{
        &self.id
    }
    fn title(&self)->&str{
        &self.title
    }
    fn topic_id(&self)->&str{
        &self.topic_id
    }
    fn style(&self)->&str{
        &self.style
    }
   fn from_string(string:&str)->Self{
        let mut res = Self{title:"Oh no! I forgot to put the title in.".to_string(),id:"".to_string(),style:"default".to_owned(),topic_id:"misc".to_owned()};
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
                "topic_id"=>res.topic_id = value.split('"').nth(1).expect("ID must be contained between 2 quotation marks!").to_string(),
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
struct Markdown{
    //elements:Box<[MarkdownElement]>,
    html:Vec<String>,
    words:usize,
}
impl Markdown{
    fn from_string(string:&str)->Self{
        let html = vec![markdown::to_html(string)];
        //println!("Begun count");
        let words = string.split(|c:char| c.is_whitespace()).map(|word|(word.len() > 0) as usize).sum::<usize>();
        //println!("End count");
        Self{html,words}
    }
    fn join(&mut self,other:Self){
        self.html.extend(other.html);
        self.words += other.words;
    }
    fn length(&self)->usize{
        self.html.iter().map(|string|string.len()).sum::<usize>()
    }
    fn words(&self)->usize{
        self.words
    }
    fn to_html(&self)->String{
        let mut res = String::with_capacity(self.length() + self.html.len()*30);
        for html in &self.html{
            res.push_str(&format!("<div class = \"paragraph\">{html}</div>"));
        }
        res
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
        
        //println!("metadata:{metadata:?},markdown:{markdown:?}");
        Ok(Self{metadata:metadata.into_iter().nth(0).unwrap(),markdown})
    }
    fn id(&self)->&str{
        self.metadata.id()
    }
    fn topic_id(&self)->&str{
        self.metadata.topic_id()
    }
    fn title(&self)->&str{
        self.metadata.title()
    }
    fn to_html(&self,articles:&[Article])->String{
        let title = self.metadata.title();
        let style = self.metadata.style();
        let head = format!("<head><meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\"><title>{title}</title><link rel=\"stylesheet\" href=\"{style}.css\"></head>");
        let words = self.markdown.words();
        let time_min = ((words as f32/350.0).floor() as usize).max(1);
        let time_max = ((words as f32/220.0).ceil() as usize).max(2);
        let article_top = format!("<h1 class=\"title\">{title}</h1><br><h6><i>{time_min} - {time_max} minute read</i></h6>");
        let article_html = self.markdown.to_html();
        let references = if self.id() == "home"{
            let mut topics = find_topics(articles);
            topics.sort_by(|(name_a,_),(name_b,_)|{name_a.cmp(name_b)});  
            let mut topics_html = String::new();
            for (topic_id,articles) in topics.iter(){
                let topic_name = topic_id.replace("_"," ");
                let mut article_list = String::new();
                for (article_id,article_name) in articles{
                    article_list.push_str(&format!("<br><a href=\"{article_id}.html\">{article_name}</a>"));
                }
                topics_html.push_str(&format!("<div class = \"topic_field\"><h3>{topic_name}{article_list}<h3></div>"));
            }
            format!("<h2>Topics, and articles</h2>{topics_html}")
        }else{
            "".to_owned()
        };
        let article_html = format!("<div class=\"article\">{article_top}{article_html}{references}</div>");
        let body = format!("<body>{article_html}</body>");
        let final_html = format!("<!DOCTYPE html><html>{head}{body}</html>");
        final_html
    }
}
fn collect_articles_from_dir(path:&Path)->std::io::Result<Vec<Article>>{
    let mut articles = Vec::new();
    for file in std::fs::read_dir(path)?{
        let file = file?;
        if file.file_type()?.is_file() && file.path().extension().is_some_and(|extension| extension == "fat_md"){
            let file = std::fs::File::open(file.path())?;
            let article = Article::from_file(file)?;
            articles.push(article);
        }
        //println!("{}", file.unwrap().path().display());
    }
    Ok(articles)
}
fn write_articles(out_dir:&Path,articles:&[Article])->std::io::Result<()>{
    //let out_dir = PathBuf::from(out_dir);
    std::fs::create_dir_all(&out_dir)?;
    for article in articles{
                use std::io::Write;
            let html = article.to_html(&articles);
            let mut out_path = (out_dir).to_owned();
            out_path.push(article.id());
            out_path.set_extension("html");
            let mut out = std::fs::File::create(out_path)?;
            out.write(&html.as_bytes())?;
    }
    Ok(())
}
fn find_topics(articles:&[Article])->Box<[(String,Vec<(String,String)>)]>{
    let mut topics = HashMap::with_capacity(32);
    for article in articles{
        if article.topic_id() != "hidden"{
             topics.entry(article.topic_id().to_owned()).or_insert(Vec::new()).push((article.id().to_owned(),article.title().to_owned()));
        }   
    }
    topics.into_iter().collect::<Box<[_]>>()
}
fn main() {
    let args = Args::parse();
    let articles = collect_articles_from_dir(&args.path).unwrap();
    let topics = find_topics(&articles);
    write_articles(Path::new("../generated_html"),&articles).unwrap();
    println!("Hello, world {args:?}!");
}
