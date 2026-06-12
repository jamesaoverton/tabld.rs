# TabLD -- Tabular Linked Data

TabLD is a concrete syntax for RDF
that can be inserted into a relational database,
side-by-side with other tabular data.
The goal is to make it easier to work with RDF and OWL data
in existing systems and with familiar tools.

The current alpha version is focused on RDF/XML files
that are output from the OWLAPI,
because this is the most common use case
when working with OBO projects.
It will eventually handle a range of RDF and OWL formats.



## Examples

These RDF triples:

```
<http://example.com/b> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/2002/07/owl#Class> .
<http://example.com/b> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <http://example.com/a> .
<http://example.com/b> <http://www.w3.org/2000/01/rdf-schema#label> "B"^^<http://www.w3.org/2001/XMLSchema#string> .
```

become these TabLD rows:

subject | predicate | object | datatype | annotations
---|---|---|---|---
http://example.com/b | http://www.w3.org/1999/02/22-rdf-syntax-ns#type | http://www.w3.org/2002/07/owl#Class | _ID |
http://example.com/b | http://www.w3.org/2000/01/rdf-schema#subClassOf | http://example.com/a | _ID | 
http://example.com/b | http://www.w3.org/2000/01/rdf-schema#label> | B | http://www.w3.org/2001/XMLSchema#string |

Nested RDF structures such as lists and nested objects
are grouped together and represented with JSON-LD.
So this OWL class expression "b subClassOf (r some a)"
is represented by these triples using blank nodes:

```
<http://example.com/b> <http://www.w3.org/2000/01/rdf-schema#subClassOf> _:b1 .
_:b1 <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://www.w3.org/2002/07/owl#Restriction> .
_:b1 <http://www.w3.org/2002/07/owl#onProperty> <http://example.com/r> .
_:b1 <http://www.w3.org/2002/07/owl#someValuesFrom> <http://example.com/a> .
```

where `_:b1` becomes this JSON-LD:

```json
{
    "http://www.w3.org/1999/02/22-rdf-syntax-ns#type": [
        {"@id": "http://www.w3.org/2002/07/owl#Restriction"}
    ],
    "http://www.w3.org/2002/07/owl#onProperty": [
        {"@id": "http://example.com/r"}
    ],
    "http://www.w3.org/2002/07/owl#someValuesFrom": [
        {"@id": "http://example.com/a"}
    ]
}
```

For TabLD we canonicalize this JSON
following RFC 8785: JSON Canonicalization Scheme (JCS)
and also treating arrays of RDF objects as sets.gh

```json
{"http://www.w3.org/1999/02/22-rdf-syntax-ns#type":[{"@id":"http://www.w3.org/2002/07/owl#Restriction"}],"http://www.w3.org/2002/07/owl#onProperty":[{"@id":"http://example.com/r"}],"http://www.w3.org/2002/07/owl#someValuesFrom":[{"@id":"http://example.com/a"}]}
```

Then we insert the JSON into the "object" column
and give it the "_JSONLD" datatype.

subject | predicate | object | datatype | annotations
---|---|---|---|---
http://example.com/b | http://www.w3.org/2000/01/rdf-schema#subClassOf | {"http://www.w3.org/1999/02/22-rdf-syntax-ns#type":[{"@id":"http://www.w3.org/2002/07/owl#Restriction"}],"http://www.w3.org/2002/07/owl#onProperty":[{"@id":"http://example.com/r"}],"http://www.w3.org/2002/07/owl#someValuesFrom":[{"@id":"http://example.com/a"}]} | _JSONLD | 


## Previous Work

TabLD is very similar to LDTab,
except that it does not use CURIEs in the primary storage,
and it uses JSON-LD expanded syntax for objects and annotations
instead of LDTab's custom JSON format.
Although IRIs take up more space than CURIEs
and are harder for humans to read,
the key advantages is consistent encoding,
regardless of variations in the choices of prefixes.
Once the choice is made to use IRIs throughout,
it then makes more sense to use the standardized JSON-LD expanded syntax
rather than a custom JSON syntax.

TabLD and LDTab grew out of RDFTab,
with the key difference that nested lists and objects
are collected into nested JSON structures.
This vastly reduces the number of blank nodes,
and makes it easier to work with complex statements.
