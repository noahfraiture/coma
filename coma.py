import argparse
from selenium import webdriver
from selenium.webdriver.chrome.service import Service
from bs4 import BeautifulSoup, Comment
from urllib.parse import urljoin, urlparse
from colorama import Fore


def fetch_html(url):
    driver.get(url)
    html = driver.page_source
    return html


def extract_comments(soup):
    comments = soup.find_all(string=lambda text: isinstance(text, Comment))
    return comments


def extract_text(soup):
    texts = soup.stripped_strings
    return texts


# FIXME : don't takes all link
def extract_links(soup, base_url):
    links = set()
    for link in soup.find_all("a", href=True):
        url = urljoin(base_url, link["href"])
        if is_same_domain(base_url, url):
            links.add(url)
    return links


def is_same_domain(base_url, link):
    base_domain = urlparse(base_url).netloc
    link_domain = urlparse(link).netloc
    return base_domain == link_domain or not link_domain


def green(text):
    return Fore.GREEN + str(text) + Fore.RESET


def main(url, directories, extract_type):
    # Initialize WebDriver
    global driver
    options = webdriver.ChromeOptions()
    options.add_argument("--headless")
    driver = webdriver.Chrome(options=options)

    visited_urls = set()
    urls_to_visit = set([url])

    for directory in directories:
        urls_to_visit.add(urljoin(url, directory))

    while urls_to_visit:
        current_url = urls_to_visit.pop()
        if current_url in visited_urls:
            continue

        visited_urls.add(current_url)
        print(f"Visiting: {current_url}")

        html = fetch_html(current_url)
        soup = BeautifulSoup(html, "html.parser")

        if extract_type == "comments":
            items = extract_comments(soup)
            print(f"Found {green(len(items))} comments in {green(current_url)}:")
        elif extract_type == "text":
            items = list(extract_text(soup))
            print(f"Found {green(len(items))} text items in {green(current_url)}:")
        elif extract_type == "links":
            items = extract_links(soup, url)
            print(f"Found {green(len(items))} links in {green(current_url)}:")

        for item in items:
            print(item)
        print()

        if extract_type == "links":
            urls_to_visit.update(items)

    driver.quit()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Search for HTML comments, text, or links on a website.")
    parser.add_argument("url", type=str, help="The base URL to start searching from")
    parser.add_argument("directories", type=str, nargs="*", help="A list of directories to search in")
    parser.add_argument(
        "--extract",
        type=str,
        choices=["comments", "text", "links"],
        required=True,
        help="Type of data to extract: comments, text, or links",
    )

    args = parser.parse_args()

    main(args.url, args.directories, args.extract)
