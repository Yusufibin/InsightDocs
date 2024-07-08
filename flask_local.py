import os
import threading
from flask import Flask, render_template, request, jsonify
import webview  

app = Flask(__name__, template_folder='./')

# Fonction pour la simulation
def rechercher_documents(requete, top_k=5):
 
    resultats = [
        {"phrase": " la premiere partie du document.", "fichier": "document1.docx", "similarite": 0.95},
        {"phrase": "Une autre phrase pertinente trouvée dans un document.", "fichier": "document2.docx", "similarite": 0.82},
    ]
    return resultats[:top_k]

#page principale 
@app.route('/', methods=['GET']) 
def index():
    return render_template('index.html')


#page pour la recherche. 
@app.route('/search/', methods=['POST']) 
def search():
    requete = request.form.get('requete')
    resultats = rechercher_documents(requete)
    return jsonify(resultats)


#def qa():
#    pass

#def settings():        

def start_server():
    app.run(host='127.0.0.1', port=5000, debug=True)

if __name__ == '__main__':
   
    t = threading.Thread(target=start_server)
    t.daemon = True
    t.start()
    
    webview.create_window("Recherche de documents", "http://127.0.0.1:5000/")
    webview.start()