// File: frontend/lib/api.ts

import { GraphQLClient } from 'graphql-request';

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080/graphql';

const client = new GraphQLClient(API_URL);

export async function fetchCollections() {
  // TODO: Implement fetchCollections query
}

export async function fetchArticle(slug: string) {
  console.log(slug);
  console.log(client);
  // TODO: Implement fetchArticle query
}

// TODO: Add more API functions as needed