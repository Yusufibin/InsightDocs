import os
import threading
from flask import Flask, render_template, request, jsonify
import webview 

app = Flask(__name__, template_folder='templates')

# Fonction de simulation de recherche
def rechercher_documents(requete, top_k=5):
    
  
    resultats = [
        {"phrase": "la premiere partie du document.", "fichier": "document1.docx", "similarite": 0.95},
        {"phrase": "Une autre phrase pertinente trouvée dans un document.", "fichier": "document2.docx", "similarite": 0.82},
    ]
    return resultats[:top_k]

# Page d'accueil
@app.route('/', methods=['GET']) 
def index():
    return render_template('index.html')


@app.route('/search/', methods=['POST']) 
def search():
    requete = request.form.get('requete')
    resultats = rechercher_documents(requete)
    return jsonify(resultats)

 
def start_server():
    app.run(host='127.0.0.1', port=5000, debug=False)  

if __name__ == '__main__':
    
    server_thread = threading.Thread(target=start_server)
    server_thread.daemon = True 
    server_thread.start()

    
    webview.create_window("Recherche de documents", "http://127.0.0.1:5000/")
    webview.start() 
