# Résumé
***level:*** user goal
***title:*** Régler le niveau de batterie minimum (seuil)
***user:*** Utilisateur de l'ordinateur 
***but:*** Choisir à quel pourcentage de batterie l'ordinateur portable entrera en hibernation

# Contexte
***preconditions:*** 
- La fenêtre de configuration de l'application est visible (UC-04)
- Si aucune valeur prééxistante n'existe pas (1er lancement par exemple), la valeur par défaut est 10%
***success guarantee:*** La valeur choisie par l'utilisateur est enregistrée même si l'application se ferme et est relancée
***minimal guarantee:*** La valeur précédemment choisie n'est pas modifiée


# Scénario primaire
1. 

# Extensions

# Not implemented yet
- Moment de l'enregistrement (à préciser dans le scénario primaire)
- Changement pendant que UC-02 tourne : le seuil bouge pendant que la surveillance est active — effet immédiat ? Ça mérite au moins une phrase dans UC-02 ou une SR partagée.
- bornes de validité du seuil (min ? max ? pas de 0 %) 