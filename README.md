# Coma - Website Scraper

## Overview
Coma is a lightweight command-line tool designed for scraping various types of content from web pages, such as text, comments, links, and images. Its simplicity and flexibility make it easy for users to extract the specific data they need from a given URL.

![Logo shrimp](shrimp.jpg)

## Program Behavior

### Command Structure
To use Coma, the basic command structure is as follows:

```
coma [OPTIONS] --url <URL> <COMMAND>
```

Where `<URL>` is the website you want to scrape, and `<COMMAND>` specifies what type of data you wish to extract.

### Commands
The available commands enable you to target specific content on the web page:

- **texts**: Extracts the text present in the HTML of the page.
- **comments**: Extracts any comments found in the HTML (such as those in HTML comment tags).
- **links**: Extracts all hyperlinks from the page, allowing you to see the navigation structure or related pages.
- **images**: Extracts the URLs of images present on the page.
- **help**: Displays the help menu, providing information on usage and available options.

### Options
Coma includes several options to customize its behavior:

- `-u, --url <URL>`: Mandatory option to specify the URL to start the scraping process.
- `-d, --depth <DEPTH>`: Determines how deep the scraper should go from the specified URL:
  - `0`: Scrapes only the specified URL.
  - `<0`: Enables infinite depth, allowing the scraper to traverse through all linked pages.
  - Default is `0`.
  
- `-b, --bound <BOUND>`: Sets a filter to include only URLs containing a specific substring. This can be useful for limiting the scraping to a specific domain or section of a website. The default value is an empty string, meaning no filtering is applied.
  
- `-t, --thread <THREAD>`: Sets the maximum number of concurrent asynchronous calls to be made during scraping. The default is set to `5`, which balances speed and performance without overwhelming the target server.
  
- `-h, --help`: Prints the help menu for Coma, including usage instructions and command options.
  
- `-V, --version`: Displays the current version of Coma.

### Example Usage
To illustrate how Coma works, here are a few example commands:

1. Extract all text from a single web page:
   ```
   coma -u https://example.com texts
   ```

2. Extract all links from a website while allowing for a depth of 1:
   ```
   coma -u https://example.com -d 1 links
   ```

3. Scrape images from a webpage with specific URL filtering:
   ```
   coma -u https://example.com/jobs -b example.com/jobs images
   ```

4. Display the help menu:
   ```
   coma help
   ```

## Plan for the Future

### Topology 
I aim to provide the complete topology of the website based on different heuristics:
- Hierarchy of the website.
- Discovery from the provided link using BFS (Breadth-First Search) and DFS (Depth-First Search).

There are different ways to represent this graph:
- ASCII representation within the terminal.
- Image rendering in the terminal (covering various protocols, though not all terminals support them).
- HTML page for a dynamic topology similar to what Neo4j provides.

### Commands 
We could add more command options beyond the current selection:
- Forms
- Full html page
- Regex patterns inside the texts with some useful preset

### Options
It's important to improve the usability of the tool with these options :
- Output of different format, it would be useful to have csv, json and maybe more
- Proxy
- Cookies and header

## Conclusion
Coma is a flexible and straightforward tool for anyone needing to scrape data from websites quickly. Users can easily customize their scraping experience through various commands and options, making it suitable for a wide range of web data extraction tasks.
