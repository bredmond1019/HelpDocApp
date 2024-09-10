# embedding_service.py

import os
from flask import Flask, request, jsonify
from sentence_transformers import SentenceTransformer
import numpy as np
import logging

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

if __name__ == '__main__':
    app.run(debug=True, port=5000)