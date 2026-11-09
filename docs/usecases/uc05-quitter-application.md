# Résumé
***level:*** user goal
***title:*** Quitter l'application 
***user:*** Utilisateur de l'ordinateur
***but:*** Quitter l'application 

# Contexte
***preconditions:*** L'application est lancée en mode barre d'état système
***success guarantee:*** 
- L'application n'est plus exécutée
- Si l'application est lancée au démarrage (UC-03), elle redémarrera selon ces conditions
***minimal guarantee:*** Si l'arrêt échoue, l'application continue de tourner et l'erreur est inscrite dans un journal


# Scénario primaire
1. L'utilisateur clique droit sur l'icône dans la barre d'état
2. L'application affiche un menu avec l'option "Quitter Soporifique" et une icône parlant
3. L'utilisateur clique sur cette option
4. L'application affiche une demande de confirmation : "Voulez-vous vraiment quitter Soporifique ? La surveillance de la batterie sera arrêtée."
5. L'utilisateur clique sur "Oui"
6. L'application se ferme et disparaît de la barre d'état système

# Extensions
2a. 
    1. L'utilisateur clique droit sur l'icône ou l'utilisateur clique (droit ou gauche) n'importe où ailleurs
    2. Le menu disparaît et l'application continue de fonctionner

5a.
    1. L'utilisateur clique sur "Non"
    2. La demande de confirmation disparait, l'application reste ouverte
