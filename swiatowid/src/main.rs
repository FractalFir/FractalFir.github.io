use clap::*;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
#[derive(Debug, Parser)]
struct Args {
    path: PathBuf,
}
#[derive(Debug)]
struct Metadata {
    title: String,
    id: String,
    style: String,
    category: String,
    date: Option<String>,
}
impl Metadata {
    fn id(&self) -> &str {
        &self.id
    }
    fn title(&self) -> &str {
        &self.title
    }
    fn category(&self) -> &str {
        &self.category
    }
    fn style(&self) -> &str {
        &self.style
    }
    fn date(&self) -> Option<&String> {
        self.date.as_ref()
    }
    fn from_string(string: &str) -> Self {
        let mut res = Self {
            title: "Oh no! I forgot to put the title in.".to_string(),
            id: "".to_string(),
            style: "default".to_owned(),
            category: "hidden".to_owned(),
            date: None,
        };
        for line in string.lines() {
            // There are no characters that are not whitespace(thus all are whitespace), skip.
            if !line.chars().any(|c| !c.is_whitespace()) {
                continue;
            }
            let (variable, value) = {
                let mut tmp = line.split('=');
                let variable = tmp
                    .next()
                    .expect("Syntax error! Line did not set a variable!");
                let value = tmp.next().expect("Syntax error! Variable set to nothing!");
                let variable = variable.trim();
                (variable, value)
            };
            match variable {
                "title" => {
                    res.title = value
                        .split('"')
                        .nth(1)
                        .expect("Title must be contained between 2 quotation marks!")
                        .to_string()
                }
                "id" => {
                    res.id = value
                        .split('"')
                        .nth(1)
                        .expect("ID must be contained between 2 quotation marks!")
                        .to_string()
                }
                "category" => {
                    res.category = value
                        .split('"')
                        .nth(1)
                        .expect("ID must be contained between 2 quotation marks!")
                        .to_string()
                }
                "date" => {
                    res.date = Some(
                        value
                            .split('"')
                            .nth(1)
                            .expect("ID must be contained between 2 quotation marks!")
                            .to_string(),
                    )
                }
                _ => panic!(
                    "unknown variable \"{var}\"!",
                    var = variable.escape_debug().to_string()
                ),
            }
            //println!("line:{line}");
        }
        if res.id.is_empty() {
            panic!("No unique article id!");
        }
        res
    }
}
#[derive(Debug)]
struct Markdown {
    //elements:Box<[MarkdownElement]>,
    html: Vec<String>,
    words: usize,
}
impl Markdown {
    fn from_string(string: &str) -> Self {
        let html = markdown::to_html(string);
        let html = html.replace("&#8217;", "'"); //.replace("<code","<div class = \"code\"><code").replace("</code","</div></code");
        let html = vec![html];
        //println!("Begun count");
        let words = string
            .split(|c: char| c.is_whitespace())
            .map(|word| (word.len() > 0) as usize)
            .sum::<usize>();
        //println!("End count");

        Self { html, words }
    }
    fn join(&mut self, other: Self) {
        self.html.extend(other.html);
        self.words += other.words;
    }
    fn length(&self) -> usize {
        self.html.iter().map(|string| string.len()).sum::<usize>()
    }
    fn words(&self) -> usize {
        self.words
    }
    fn to_html(&self) -> String {
        let mut res = String::with_capacity(self.length() + self.html.len() * 30);
        for html in &self.html {
            res.push_str(&format!("<div class = \"paragraph\">{html}</div>"));
        }
        res
    }
}
struct Article {
    metadata: Metadata,
    markdown: Markdown,
}
impl Article {
    fn from_file(mut file: impl Read) -> std::io::Result<Self> {
        let mut content = String::with_capacity(0x1000);
        file.read_to_string(&mut content)?;

        let mut metadata_start: Vec<_> = (&content)
            .match_indices("<metadata>")
            .map(|(i, _)| i + "<metadata>".len())
            .collect();
        let mut metadata_end: Vec<_> = (&content)
            .match_indices("</metadata>")
            .map(|(i, _)| i)
            .collect();
        let metadata: Vec<_> = metadata_start
            .iter()
            .zip(metadata_end.iter())
            .map(|(start, end)| &content[(*start)..(*end)])
            .map(|string| Metadata::from_string(string))
            .collect();

        let mut markdown_start: Vec<_> = (&content)
            .match_indices("<markdown>")
            .map(|(i, _)| i + "<metadata>".len())
            .collect();
        let mut markdown_end: Vec<_> = (&content)
            .match_indices("</markdown>")
            .map(|(i, _)| i)
            .collect();

        let mut markdown = markdown_start
            .iter()
            .zip(markdown_end.iter())
            .map(|(start, end)| &content[(*start)..(*end)])
            .map(|string| Markdown::from_string(string)); //.collect();
        let mut first_markdown = markdown
            .next()
            .expect("Expected at least one markdown tag!");
        for next_markdown in markdown {
            first_markdown.join(next_markdown);
        }
        let markdown = first_markdown;
        assert_eq!(
            metadata.len(),
            1,
            "Expected 1 and exactly one metadata field!"
        );

        //println!("metadata:{metadata:?},markdown:{markdown:?}");
        Ok(Self {
            metadata: metadata.into_iter().nth(0).unwrap(),
            markdown,
        })
    }
    fn id(&self) -> &str {
        self.metadata.id()
    }
    fn category(&self) -> &str {
        self.metadata.category()
    }
    fn title(&self) -> &str {
        self.metadata.title()
    }
    fn link(&self) -> String {
        format!("{}.html", self.id())
    }
    fn to_html(&self, articles: &[Article]) -> String {
        let title = self.metadata.title();
        let style = self.metadata.style();
        let hljs_cil = include_str!("hljs_cil.js");
        let hljs = format!("<link rel=\"stylesheet\" href=\"https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/a11y-dark.min.css\" media=\"none\" onload=\"if(media!='all')media='all'\">
<script async src=\"https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js\"></script>
<!-- and it's easy to individually load additional languages -->
<script async src=\"https://unpkg.com/highlightjs-copy/dist/highlightjs-copy.min.js\"></script>
<script>window.addEventListener('load', () => {{
const start = Date.now();
hljs.addPlugin(new CopyButtonPlugin());
{hljs_cil}\nhljs.highlightAll();\n
const end = Date.now();
console.log(`Highlight time: ${{end - start}} ms`);
}});</script>");
        let head = format!("<head><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{title}</title><link rel=\"stylesheet\" href=\"{style}.css\">{hljs}</head>");
        let words = self.markdown.words();
        let time_min = ((words as f32 / 350.0).floor() as usize).max(1);
        let time_max = ((words as f32 / 220.0).ceil() as usize).max(2);
        let date = if let Some(date) = self.metadata.date() {
            format!("Published on {date}<br>")
        } else {
            "".into()
        };
        let article_top = format!("<div class = \"article_header\"><h1 class=\"title\">{title}</h1><br><small><div class = \"article_metadata\">{date}<i>{time_min} - {time_max} minute read</i></div></small></div>");
        let article_html = self.markdown.to_html();
        let references = if self.id() == "home" {
            let mut categories = find_categories(articles);
            categories.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));
            let mut categories_html = String::new();
            for (category, articles) in categories.iter() {
                assert!(!category.is_empty());
                let category_name = category.replace("_", " ");
                println!("category_name:\"{category_name}\"");
                let mut article_list = String::new();
                for (article_id, article_name) in articles {
                    article_list.push_str(&format!(
                        "<br><a href=\"{article_id}.html\">{article_name}</a>"
                    ));
                }
                categories_html.push_str(&format!(
                    "<div class = \"category_field\"><h3>{category_name}<h3>{article_list}</div>"
                ));
            }
            format!("<h2>Categories, and articles in them.</h2>{categories_html}")
        } else {
            "".to_owned()
        };
        let article_html =
            format!("{article_top}<div class=\"article\">{article_html}{references}</div>");
        let github = "https://www.github.com/FractalFir";
        let reddit = "https://www.reddit.com/user/FractalFir";
        let linked_in = "https://www.linkedin.com/in/micha%C5%82-kostrubiec-85a037269/";
        let rss = "https://fractalfir.github.io/generated_html/rss.xml";
        let navigation = format!("<div class = \"nav_container\"><nav class=\"topnav\">
            <b><a class=\"active\" href=\"./home.html\">Home</a></b>
            <a href=\"{github}\"><img src = \"../images/github-mark-white.svg\" class = \"github_link\" width = \"25\" height = \"25\" alt = \"Link to my github account.\"></a>
            <a href=\"{reddit}\"><img src = \"../images/Reddit_Mark_OnWhite.svg\" class = \"reddit_link\" width = \"27.5\" height = \"27.5\" alt = \"Link to my reddit account.\"></a>
            <a href=\"{linked_in}\"><img src = \"../images/LI-In-Bug.png\" class = \"linked_id_link\" height = \"27.5\" alt = \"Link to my linkedin account.\"></a>
            <a href=\"{rss}\"><img src = \"https://upload.wikimedia.org/wikipedia/en/4/43/Feed-icon.svg\" class = \"rss_link\" height = \"27.5\" alt = \"Link to my rss feed.\"></a>
        </nav></div>");
        let body = format!("<body>{navigation}{article_html}</body>");
        let final_html = format!("<!DOCTYPE html><html lang =\"en\">{head}{body}</html>");
        final_html
    }
}
fn collect_articles_from_dir(path: &Path) -> std::io::Result<Vec<Article>> {
    let mut articles = Vec::new();
    for file in std::fs::read_dir(path)? {
        let file = file?;
        if file.path().is_dir() {
            articles.extend(collect_articles_from_dir(&file.path())?);
            continue;
        }
        if file.file_type()?.is_file()
            && file
                .path()
                .extension()
                .is_some_and(|extension| extension == "fat_md")
        {
            println!("Parsing article {:?}...", file.path());
            let file = std::fs::File::open(file.path())?;

            let article = Article::from_file(file)?;
            articles.push(article);
        }
        //println!("{}", file.unwrap().path().display());
    }
    Ok(articles)
}
fn write_rss(out_dir: &Path, articles: &[Article]) -> std::io::Result<()> {
    std::fs::create_dir_all(&out_dir)?;
    let mut out_path = PathBuf::from(out_dir);
    let mut rss = format!("<?xml version=\"1.0\" encoding=\"UTF-8\" ?><rss version=\"2.0\">");
    let mut items = String::new();
    for article in articles {
        let title = article.title();
        if article.category() == "hidden" {
            continue;
        }
        let date = article
            .metadata
            .date()
            .expect("All articles must have dates!");
        let link = article.link();
        items.push_str(&format!("<item><title>{title}</title><pubDate>{date}</pubDate><link>https://fractalfir.github.io/generated_html/{link}</link><guid>https://fractalfir.github.io/generated_html/{link}</guid></item>"));
    }
    let main_chanel = format!(
        "
<channel>
    <title>Fractal Fir's blog</title>
    <description>A blog I which I document my coding adventures.</description>
    <link>https://fractalfir.github.io/generated_html/home.html</link>
    <ttl>720</ttl>
    {items}
</channel>
"
    );
    rss.push_str(&main_chanel);
    rss.push_str(&"</rss>");
    out_path.push("rss");
    out_path.set_extension("xml");
    std::fs::File::create(out_path)?.write_all(rss.as_bytes())
}
fn write_articles(out_dir: &Path, articles: &[Article]) -> std::io::Result<()> {
    //let out_dir = PathBuf::from(out_dir);
    std::fs::create_dir_all(&out_dir)?;
    for article in articles {
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
fn find_categories(articles: &[Article]) -> Box<[(String, Vec<(String, String)>)]> {
    let mut categories = HashMap::with_capacity(32);
    for article in articles {
        if article.category() != "hidden" {
            categories
                .entry(article.category().to_owned())
                .or_insert(Vec::new())
                .push((article.id().to_owned(), article.title().to_owned()));
        }
    }
    categories.into_iter().collect::<Box<[_]>>()
}
fn main() {
    let args = Args::parse();
    let articles = collect_articles_from_dir(&args.path).unwrap();
    let categories = find_categories(&articles);
    write_articles(Path::new("../generated_html"), &articles).unwrap();
    write_rss(Path::new("../generated_html"), &articles).unwrap();
    println!("Hello, world {args:?}!");
}
