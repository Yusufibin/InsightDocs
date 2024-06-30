use std::fs;
use std::path::Path;

use faiss::{Index, IndexFlatIP, MetricType};
use mongodb::{Client, Collection};
use serde::{Deserialize, Serialize};
use transformers::{AutoModel, AutoTokenizer, pipelines::embeddings};

#[derive(Debug, Serialize, Deserialize)]
struct Document {
    file_path: String,
    text: String,
    embedding: Vec<f32>,
}

// Configuration MongoDB
const MONGODB_URI: &str = "mongodb://localhost:27017/";
const DATABASE_NAME: &str = "semantic_search";
const COLLECTION_NAME: &str = "documents";

// Configuration du modèle de plongement
const MODEL_NAME: &str = "sentence-transformers/all-mpnet-base-v2";

// Configuration Faiss
const FAISS_INDEX_PATH: &str = "documents.index";

async fn connect_to_mongodb() -> mongodb::error::Result<Collection<Document>> {
    let client = Client::with_uri_str(MONGODB_URI).await?;
    let db = client.database(DATABASE_NAME);
    let collection = db.collection(COLLECTION_NAME);
    Ok(collection)
}

fn get_text_files(folder_path: &str) -> Vec<String> {
    let mut text_files = Vec::new();
    for entry in fs::read_dir(folder_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .map_or(false, |ext| ["txt", "md", "pdf"].contains(&ext.to_str().unwrap()))
        {
            text_files.push(path.to_str().unwrap().to_string());
        }
    }
    text_files
}

fn embed_text(
    tokenizer: &AutoTokenizer,
    model: &AutoModel,
    text: &str,
) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let embeddings = embeddings::pipeline(
        "feature-extraction",
        model,
        tokenizer,
        None,
        None,
        None,
        false,
        None,
    )
    .unwrap()
    .predict(&[text], None, 32)?;

    Ok(embeddings[0].get(0).unwrap().to_owned().try_into()?)
}

fn create_index(documents: &[Document]) -> Index {
    let dim = documents[0].embedding.len();
    let mut index = IndexFlatIP::new(dim).unwrap();

    let embeddings: Vec<_> = documents
        .iter()
        .map(|doc| doc.embedding.as_slice())
        .collect();
    index
        .add(embeddings.as_slice())
        .expect("Erreur lors de l'ajout des vecteurs à l'index");
    index
}

fn save_index(index: &Index, path: &str) {
    index.save(Path::new(path)).unwrap();
}

fn load_index(path: &str) -> Index {
    Index::load(Path::new(path)).unwrap()
}

fn search(
    query: &str,
    index: &Index,
    documents: &[Document],
    top_k: usize,
    tokenizer: &AutoTokenizer,
    model: &AutoModel,
) -> Result<Vec<(usize, f32)>, Box<dyn std::error::Error>> {
    let query_embedding = embed_text(tokenizer, model, query)?;
    let k = top_k.min(index.ntotal());
    let (distances, indices) = index.search(&[query_embedding], k)?;
    Ok(indices[0]
        .iter()
        .zip(distances[0].iter())
        .map(|(i, d)| (*i as usize, *d))
        .collect())
}

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let documents_collection = connect_to_mongodb().await?;
    let tokenizer = AutoTokenizer::from_pretrained(MODEL_NAME, None)?;
    let model = AutoModel::from_pretrained(MODEL_NAME)?;

    // Indexation initiale (si nécessaire)
    if documents_collection.count_documents(None, None).await? == 0 {
        println!("Indexation des documents...");
        let desktop_files = get_text_files(os::path::expand_user("~/Desktop").to_str().unwrap());
        let documents_files = get_text_files(os::path::expand_user("~/Documents").to_str().unwrap());
        let all_files = desktop_files.iter().chain(documents_files.iter());

        let mut documents: Vec<Document> = Vec::new();
        for file_path in all_files {
            let text = fs::read_to_string(file_path).unwrap();
            let embedding = embed_text(&tokenizer, &model, &text).unwrap();
            documents.push(Document {
                file_path: file_path.to_string(),
                text,
                embedding,
            });
        }
        documents_collection.insert_many(documents.clone(), None).await?;

        let index = create_index(&documents);
        save_index(&index, FAISS_INDEX_PATH);
        println!("Indexation terminée.");
    } else {
        println!("Chargement de l'index existant...");
        let _index = load_index(FAISS_INDEX_PATH);
        println!("Index chargé.");
    }

    // Boucle de recherche
    loop {
        let mut query = String::new();
        println!("Entrez votre requête (ou 'q' pour quitter) : ");
        std::io::stdin().read_line(&mut query).unwrap();
        let query = query.trim();

        if query == "q" {
            break;
        }

        let index = load_index(FAISS_INDEX_PATH);
        let all_documents: Vec<Document> = documents_collection
            .find(None, None)
            .await?
            .map(|doc| doc.unwrap())
            .collect();
        let results = search(query, &index, &all_documents, 5, &tokenizer, &model)?;

        println!("Résultats de la recherche :");
        for (i, score) in results {
            let document = &all_documents[i];
            println!(
                "- {} (Score: {})",
                document.file_path, score
            );
        }
    }
    Ok(())
}
