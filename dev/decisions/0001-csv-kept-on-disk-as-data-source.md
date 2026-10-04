# 0001. Read the data from a CSV file the user keeps on disk

- Status: Accepted
- Date: 2026-10-04

## Context

The tool needs the published value of every indicator in every mesh of a region.
The MLIT Data Platform offers that in two forms for each region, and an API beside them.

- Each record of the Urban QOL catalogue carries one CSV file to download and the URL of a vector tile set on the platform's S3 bucket (the platform's own search response for the catalogue, checked 2026-10-04).
- The CSV file can be downloaded from the record's page in a browser without an account (done for two prefectures, 2026-10-04).
  - It has one row for each mesh and indicator, with the indicator's name in the row (the two files, checked 2026-10-04).
- When the download is asked for, the record's page calls the platform's API and the download follows, and that API answers a request without a key with `Invalid API Key.` (both observed in the browser, 2026-10-04).
- The platform's [terms of use](https://data-platform.mlit.go.jp/assets/policy/%E5%9B%BD%E5%9C%9F%E4%BA%A4%E9%80%9A%E3%83%87%E3%83%BC%E3%82%BF%E3%83%97%E3%83%A9%E3%83%83%E3%83%88%E3%83%95%E3%82%A9%E3%83%BC%E3%83%A0%E5%88%A9%E7%94%A8%E8%A6%8F%E7%B4%84.pdf) (revised September 2023) and its [API terms](https://data-platform.mlit.go.jp/assets/policy/%E5%9B%BD%E5%9C%9F%E4%BA%A4%E9%80%9A%E3%83%87%E3%83%BC%E3%82%BF%E3%83%97%E3%83%A9%E3%83%83%E3%83%88%E3%83%95%E3%82%A9%E3%83%BC%E3%83%A0API%E6%A9%9F%E8%83%BD%E5%88%A9%E7%94%A8%E8%A6%8F%E7%B4%84.pdf) (in force since 2023-09-25) say nothing of the tiles, of the bucket, of automated access, or of request rates (read 2026-10-04; articles 18 and 19 of the terms of use, on delegation and jurisdiction, were not read).
  - What could reach a program that reads the tiles is article 6 (4) of the terms of use, which forbids acts that hinder, or may hinder, the service's management and operation.
  - So the terms neither permit nor forbid reading the tiles from the bucket.
- The maintainer set the condition for reading the tiles before the terms were read: the tool reads them only where the terms allow it, and where they forbid it or do not settle it, the tool reads a CSV file kept on disk (the request this project started from, 2026-10-04).
- The platform's API needs an account and an API key, issued on agreeing to the API terms ([API introduction](https://data-platform.mlit.go.jp/api_docs/usage/introduction.html), checked 2026-10-04).
  - [mlit-dpf-mcp](https://github.com/MLIT-DATA-PLATFORM/mlit-dpf-mcp), the platform's MCP server over that API, calls itself an alpha version without guarantee, which may change or be removed without notice (its README, checked 2026-10-04).

## Decision

The tool reads the CSV file of a region from a place on the user's machine, where the user put it after downloading it from the platform in a browser.
The tool sends the platform no request for the data.

## Rejected alternatives

- **Reading the vector tiles from the platform's bucket**: the terms do not settle whether it is allowed, which is the case the maintainer's condition in the context sends to the CSV file.
- **The tool fetching the file from the record's page itself**: the page gets the download through the platform's API, which refuses a request without a key, so the tool would need the site's own key, which is passing for the site, or its user's key, which is the next alternative.
- **Downloading the file through the platform's API**: it would make an account and an API key a condition of running the tool, to save a download the user makes once per region in a browser.
- **Fetching through mlit-dpf-mcp**: it is a server for a language model's host, needs the same API key, and may change or disappear.

## Consequences

- **Dependencies added**: none.
- **Risks**:
  - The provider updates the data every five years ([the data's introduction](https://data-platform.mlit.go.jp/#/Page?id=dataintro01), checked 2026-10-04), and a changed file layout would show as a file the tool fails to read.
  - The user has a manual step before the tool shows anything, so the tool has to say what file it expects and where.
