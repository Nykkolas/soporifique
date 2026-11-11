# Résumé
***level:*** subfunction
***title:*** Accéder à la configuration actuelle
***user:*** Utilisateur de l'ordinateur
***but:*** Lire ou modifier les paramètres persistants de l'application

# Contexte
***preconditions:*** L'application est en mode barre d'état système
***success guarantee:*** 
- La fenêtre affiche les paramètres actuels et permet de les modifier (UC-01)
- L'application continue à tourner dans la barre d'état système même si la fenêtre est fermée
***minimal guarantee:*** 
- La fenêtre affiche une erreur claire s'il y a un problème
- L'utilisateur a accès au journal avec l'erreur détaillée et les raisons du problème


# Scénario primaire
1. L'utilisateur clique gauche sur l'icône de l'application dans la barre d'état
2. L'application ouvre une fenêtre dans laquelle les paramètres de l'application sont visibles et modifiables
3. L'utilisateur modifie les paramètres si il le souhaite
4. L'utilisateur clique sur un bouton "Enregistrer"
5. L'application sauvegarde les paramètres
6. L'application ferme la fenêtre de configuration

# Extensions
1a. 
    1. L'utilisateur clique droit sur l'icône en barre d'état
    2. L'application affiche un menu avec une option "Configurer..."
    3. L'utilisateur clique sur cette option
    4. L'application ferme le menu avant d'afficher la fenêtre du scénario primaire

2a. Si l'utilisateur clique gauche sur l'icône dans la barre d'état alors que la fenêtre de configuration est affichée en arrière plan, la fenêtre de paramètre revient en avant plan

2b. 
    1. Si l'utilisateur clique droit alors que la fenêtre de configuration est affichée, le menu s'affiche
    2. Si l'utilisateur choisi l'option "Configurer..."
    3. La fenêtre de configuration revient en avant plan

4a. 
    1. L'utilisateur clique sur "Annuler" ou ferme la fenêtre (par la croix par exemple)
    2. L'application ferme la fenêtre sans enregistrer les paramètres
