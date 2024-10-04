# Coma - Website Scraper

> Disclaimer: This project is currently on pause. I made some significant changes in how it's used recently (in the last merge), and it hasn't been tested much since. I plan to continue developing this tool with many great features, but for now, I am working on another project.

## Overview
Coma is a lightweight command-line tool designed for scraping various types of content from web pages, such as text, comments, links, and images. Its simplicity and flexibility make it easy for users to extract the specific data they need from a given URL.

![Logo shrimp](static/shrimp.jpg)

## Installation

You can install Coma either by compiling it locally after cloning the repository or by installing it directly from [crates.io](https://crates.io).

### Clone and Compile Locally

1. **Clone the repository:**
   ```bash
   git clone https://github.com/yourusername/coma.git
   cd coma
   ```

2. **Build the project using Cargo:**
   ```bash
   cargo build --release
   ```

3. **Run the compiled binary:**
   ```bash
   ./target/release/coma --help
   ```

### Install from crates.io

To install Coma from crates.io, use the following command:
```bash
cargo install coma
```

This will download and compile Coma, making it available for easy use from the command line.

## Program Behavior

### Command Structure
To use Coma, the basic command structure is as follows:

```
coma [OPTIONS] --url <URL> <COMMAND>
```

Where `<URL>` is the website you want to scrape, and `<COMMAND>` specifies what type of data you wish to extract.

### Commands
The available commands enable you to target specific content on the web page:

- **print**: Print the extracted content in the terminal.
- **save**: Save the extracted content in files.
- **graph**: Create an HTML topology of the website.
- **help**: Displays the help menu, providing information on usage and available options

### Options

Coma includes several options to customize its behavior:

- `-c, --content <CONTENT>`: Specifies the type of content to scrape. Available values are:
   - **texts**: Extracts the text present in the HTML of the page.
   - **comments**: Extracts any comments found in the HTML (such as those in HTML comment tags).
   - **links**: Extracts all hyperlinks from the page, allowing you to see the navigation structure or related pages.
   - **images**: Extracts the URLs of images present on the page.
   - **inputs**: Extracts input fields from forms on the page.
   - **all**: Extracts all the available types of content. (Default: all)

- `-u, --url <URL>`: Mandatory option to specify the URL to start the scraping process.
- `-d, --depth <DEPTH>`: Determines how deep the scraper should go from the specified URL:
   - `0`: Scrapes only the specified URL.
   - `<0`: Enables infinite depth, allowing the scraper to traverse through all linked pages.
   - Default is `0`.  

- `-b, --bound <BOUND>`: Sets a filter to include only URLs containing a specific substring. This can be useful for limiting the scraping to a specific domain or section of a website. The default value is an empty string, meaning no filtering is applied.
- `-t, --task <TASK>`: Sets the maximum number of concurrent asynchronous tasks to be made during scraping. The default is set to 5, which balances speed and performance without overwhelming the target server.
- `-e, --external <EXTERNAL>`: Specifies whether to include external links or not. Default is 0 (exclude external links).
- `-h, --help`: Prints the help menu for Coma, including usage instructions and command options.
- `-V, --version`: Displays the current version of Coma.

## Plan for the Future

### Topology

The current graph doesn't give the possibility to make directed link which would be great

I aim to provide the complete topology of the website based on different heuristics:
- Hierarchy of the website.
- Discovery from the provided link using BFS (Breadth-First Search) and DFS (Depth-First Search).

### Content
We could add more command options beyond the current selection:
- Full HTML page
- Regex patterns inside the texts with some useful preset
- More html tag

### Options
It's important to improve the usability of the tool with these options:
- Output of different formats, it would be useful to have CSV, JSON, and maybe more.
- Proxy
- Cookies and header
- Download the images directly

## Conclusion
Coma is a flexible and straightforward tool for anyone needing to scrape data from websites quickly. Users can easily customize their scraping experience through various commands and options, making it suitable for a wide range of web data extraction tasks.
