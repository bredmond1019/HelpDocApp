# embedding_service.py

import os
from flask import Flask, request, jsonify
from sentence_transformers import SentenceTransformer
import numpy as np
import logging

'''
This is a simple embedding service that uses the SentenceTransformer model to generate embeddings for text.

To run this service, you need to have the SentenceTransformer model installed.
You can install the model by running the following command:

conda activate embed_service
pip3 install -r requirements.txt
'''

app = Flask(__name__)

# Set up logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Initialize the model
try:
    model = SentenceTransformer('all-MiniLM-L6-v2')
    logger.info("Model loaded successfully")
except Exception as e:
    logger.error(f"Failed to load model: {str(e)}")
    raise

@app.route('/')
def index():
    return jsonify({'message': 'Welcome to the embedding service'})

@app.route('/embed', methods=['POST'])
def embed_text():
    try:
        data = request.json
        text = data.get('text', '')
        if not text:
            logger.warning("No text provided for embedding")
            return jsonify({'error': 'No text provided'}), 400
        
        # Generate embedding
        embedding = model.encode([text])[0]
        logger.info(f"Successfully generated embedding for text of length {len(text)}")
        
        return jsonify({'embedding': embedding.tolist()})
    except Exception as e:
        logger.error(f"Error generating embedding: {str(e)}")
        return jsonify({'error': f'Internal server error: {str(e)}'}), 500

@app.route('/test-embed')
def test_embed():
    try:
        test_text = "This is a test sentence for embedding."
        embedding = model.encode([test_text])[0]
        logger.info(f"Successfully generated test embedding for text: '{test_text}'")
        return jsonify({
            'test_text': test_text,
            'embedding': embedding.tolist()
        })
    except Exception as e:
        logger.error(f"Error generating test embedding: {str(e)}")
        return jsonify({'error': f'Internal server error: {str(e)}'}), 500

@app.route('/health')
def health_check():
    return jsonify({'status': 'ok'}), 200


if __name__ == '__main__':
    port = int(os.environ.get('PORT', 8080))
    logger.info(f"Starting server on port {port}")
    app.run(debug=True, port=port)
    logger.info(f"Server started successfully on port {port}")

