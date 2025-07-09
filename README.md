<div align="center">

<h3>🌍 Empowering Knowledge Access, Even Without the Internet 🌍</h3>
<h4>Search the world you’ve saved, anytime, anywhere.
</h4>

</div>

----

OfflineSearch is a robust, open-source search engine built in Rust, inspired by the simplicity and efficiency of early Google. Designed for a world where internet access can be unreliable or restricted, OfflineSearch enables users to download and index local collections of documents—HTML, XHTML, PDFs, source code, and research papers—and search them with blazing-fast, relevance-driven results. Whether you're in a region with frequent internet blackouts, a researcher preserving knowledge, or a developer working offline, OfflineSearch ensures you can find what you need, when you need it.

<br />

<div align="center"><h3>Why OfflineSearch?</h3></div>
In many parts of the world, like Iran, internet access is often disrupted, leaving individuals, students, and professionals cut off from critical information. OfflineSearch addresses this challenge by bringing the power of a search engine to your local machine. Imagine having a personal archive of downloaded web pages, academic papers, and code repositories, all searchable with precision, even during internet outages. OfflineSearch is not just a tool—it's a lifeline for knowledge access in uncertain times.

<br />

✅ **Relevance-Driven**: Uses advanced indexing and ranking algorithms inspired by early search engines to deliver the most relevant results.

✅ **Offline-First**: Works entirely on your local machine, ensuring access to your resources anytime, anywhere.

✅ **Versatile**: Supports a wide range of file types (HTML, XHTML, PDF, source code) for comprehensive search.

✅ **Fast and Efficient**: Built with Rust for unparalleled performance and memory safety.


<br />

<div align="center"><h3>Features</h3></div>

🎯 **Flexible Indexing**: Automatically crawls and indexes files in specified directories, supporting HTML, XHTML, PDFs, and plain text (e.g., source code).

🎯 **Smart Search**: Implements a ranking algorithm based on term frequency and document relevance, inspired by early PageRank concepts.

🎯 **Cross-Platform**: Runs on Windows, macOS, and Linux with minimal dependencies.

🎯 **Lightweight**: Optimized for low resource usage, making it ideal for older hardware or resource-constrained environments.

🎯 **Extensible**: Modular design allows developers to add support for new file formats or customize ranking algorithms.


<br />
<div align="center"><h3>Getting Started</h3></div>

#### Prerequisites:

- Rust (stable, version 1.65 or higher)
- Cargo (Rust's package manager)
- A local directory containing documents (HTML, XHTML, PDF, or text files)

#### Installation

1. Clone the Repository:

```bash
git clone https://github.com/lyteabovenyte/offlinesearch.git

cd offlinesearch
```


2. Build the Project:

```bash
cargo build --release
```


3. Run OfflineSearch:

```bash
cargo run --release -- --index /path/to/your/documents
```


4. Search Your Collection:

```bash
cargo run --release -- --search "your query here"
```



#### Example Usage
Index a directory of downloaded files if you haven't yet

```bash
cargo run --release -- --index ~/Documents/research_papers
```

Search for a term

```bash
cargo run --release -- --search "machine learning algorithms"
```

This will return a list of relevant documents, ranked by their relevance to your query, with snippets highlighting matching content.


<br />
<div align="center"><h3>How It Works</h3></div>

- **Crawling**: OfflineSearch scans your specified directory, parsing supported file types (HTML, XHTML, PDF, text) to extract content.
- **Indexing**: Builds an *inverted index* to map terms to documents, optimized for fast lookups and minimal memory usage.
- **Ranking**: Uses a modified **TF-IDF** (Term Frequency-Inverse Document Frequency) algorithm, inspired by early search engine techniques, to rank results by relevance.
- **Querying**: Processes natural language queries, returning results with highlighted snippets for easy navigation.


<br />
<div align="center"><h3>Future:</h3></div>

- [ ] Add support for additional file formats (e.g., Markdown, Docx).
- [ ] Implement advanced query features (e.g., boolean operators, fuzzy search).
- [ ] Create a GUI for non-technical users.
- [ ] Optimize indexing for larger datasets (>100GB).
- [ ] Add multilingual support for non-Latin scripts.

<br />
<div align="center"><h3>Why this Matters?</h3></div>

In a world where access to information is increasingly controlled or disrupted, **OfflineSearch** is a tool for empowerment. It’s for the student in a remote village, the researcher preserving knowledge, and the developer building the future, all without relying on an unstable internet connection. I aim to democratize access to information and foster a global community of knowledge seekers.