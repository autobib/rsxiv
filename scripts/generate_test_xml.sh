#!/usr/bin/env bash
set -eou pipefail

prefix="src/response/tests/"

https "export.arxiv.org/api/query?id_list=2201.13455,2201.13452,2201.13453,2201.13454" > "${prefix}query_missing_id.xml"

https "export.arxiv.org/api/query?search_query=all:electron AND all:proton" > "${prefix}query.xml"

https "export.arxiv.org/api/query?id_list=2206.06921" | sed '1s/^/r#"/; $s/$/"#/; s/^/# /' > "${prefix}query_doc.txt"

https "export.arxiv.org/api/query?id_list=1201.56789" > "${prefix}query_empty.xml"

https "export.arxiv.org/api/query?search_query=cat:math.CA AND ti:diffuse" | sed '1s/^/r#"/; $s/$/"#/; s/^/# /' > "${prefix}query_doc_long.txt"
