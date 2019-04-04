let inMemoryCache = ApolloInMemoryCache.createInMemoryCache();

let httpLink = ApolloLinks.createHttpLink(~uri="/graphql", ());

let instance = ReasonApollo.createApolloClient(~link=httpLink, ~cache=inMemoryCache, ());
