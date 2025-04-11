// Start writing your queries here.
//
// You can use the schema to help you write your queries.
//
// Queries take the form:
//     QUERY {query name}({input name}: {input type}) =>
//         {variable} <- {traversal}
//         RETURN {variable}
//
// Example:
//     QUERY GetUserFriends(user_id: String) =>
//         friends <- N<User>(user_id)::Out<Knows>
//         RETURN friends
//
//
// For more information on how to write queries,
// see the documentation at https://docs.helix-db.com
// or checkout our GitHub at https://github.com/HelixDB/helix-db

QUERY hnswinsert(vector: [Float]) =>
    AddV<Vector>(vector)
    RETURN "Success"

QUERY hnswload(vectors: [[Float]]) =>
    res <- BatchAddV<Type>(vectors)
    RETURN res::{ID}

QUERY hnswsearch(query: [Float], k: Integer) =>
    res <- SearchV<Type>(query, k)
    RETURN res

QUERY size() =>
	size <- V::COUNT
	RETURN size
