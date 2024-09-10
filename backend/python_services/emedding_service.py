# embedding_service.py

import os
from flask import Flask, request, jsonify
from sentence_transformers import SentenceTransformer
import numpy as np

app = Flask(__name__)

# Initialize the model
model = SentenceTransformer('all-MiniLM-L6-v2')

@app.route('/embed', methods=['POST'])
def embed_text():
    data = request.json
    text = data.get('text', '')
    if not text:
        return jsonify({'error': 'No text provided'}), 400
    
    # Generate embedding
    embedding = model.encode([text])[0]
    
    return jsonify({'embedding': embedding.tolist()})

if __name__ == '__main__':
    app.run(debug=True, port=5000)