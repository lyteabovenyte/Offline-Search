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

<div align="center"><h3>Features</h3></div>

🎯 **Flexible Indexing**: Automatically crawls and indexes files in specified directories, supporting HTML, XHTML, PDFs, and plain text (e.g., source code).

🎯 **Smart Search**: Implements a ranking algorithm based on [TF-IDF](https://en.wikipedia.org/wiki/Tf%E2%80%93idf), inspired by early *PageRank* concepts.

🎯 **Stemming and NLP**: Utilizes the [Snowball](https://snowballstem.org/) stemming algorithm to normalize words for more accurate matching, and incorporates basic natural language processing techniques to enable semantic search—helping you find relevant documents even when your query uses different word forms or synonyms.

🎯 **Cross-Platform**: Runs on Windows, macOS, and Linux with minimal dependencies.

🎯 **Lightweight**: Optimized for low resource usage, making it ideal for older hardware or resource-constrained environments.

🎯 **Extensible**: Modular design allows developers to add support for new file formats or customize ranking algorithms.


<br />
<div align="center"><h3>Getting Started</h3></div>

#### Prerequisites:

- Rust (stable, version 1.65 or higher)
- Cargo (Rust's package manager)
- A local directory containing documents (HTML, XHTML, PDF, or text files)

#### Usage

1. Clone the Repository:

```bash
git clone https://github.com/lyteabovenyte/offline-search.git

cd offline-search
```

2. Serve the directory:
```bash
cargo run serve [address(default: localhost:6969)] ./your_directory/ 
```

It'll start indexing the directory in the background on other thread and you can start searching via the web browser while indexing.

<br />
<div align="center"><h3>How It Works</h3></div>

- **Crawling**: OfflineSearch scans your specified directory, parsing supported file types (HTML, XHTML, PDF, text) to extract content. During crawling, terms are stemmed using the Snowball stemming algorithm and basic NLP techniques are applied to enhance semantic relevance. All processed data is cached in either a SQLite database or a JSON file for efficient retrieval.

- **Indexing**: Constructs a highly efficient *inverted index* that maps normalized terms to their occurrences across documents. Each entry in the index includes metadata such as term frequency, document location, and contextual information (e.g., title, headings, body text). This structure enables rapid, memory-efficient lookups and supports advanced search features like phrase matching and proximity queries. The indexing process also deduplicates content and filters out stop words to improve result quality.

- **Ranking**: Uses a modified **TF-IDF** (Term Frequency-Inverse Document Frequency) algorithm, inspired by early search engine techniques, to rank results by relevance.

$$
tf(t, d) = \frac{f_{t,d}}{\sum_{t' \in d} f_{t',d}}
$$

$$
\text{idf}(t, D) = \log \frac{N}{\left| \{ d \in D : t \in d \} \right|}
$$

- **Querying**: Processes natural language queries, returning semantically relevant results with highlighted snippets that match the query. The system uses the Snowball stemming algorithm to improve matching accuracy and ensure that different word forms are recognized, helping users find the most relevant information quickly.


<br />
<div align="center"><h3>Future:</h3></div>

- [ ] Add support for additional file formats (e.g., Markdown, Docx).
- [ ] Implement advanced query features (e.g., boolean operators, fuzzy search).
- [ ] Add Dynamic Web UI
- [ ] Optimize indexing for larger datasets (>100GB).
- [ ] Add multilingual support for non-Latin scripts.

<br />
<div align="center"><h3>Why this Matters?</h3></div>

In a world where access to information is increasingly controlled or disrupted, **OfflineSearch** is a tool for empowerment. It’s for the student in a remote village, the researcher preserving knowledge, and the developer building the future, all without relying on an unstable internet connection. I aim to democratize access to information and foster a global community of knowledge seekers.
