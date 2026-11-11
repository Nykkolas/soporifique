# Glossaire — langage omniprésent du domaine

Correspondance terme français (écran, specs, UC, tests) ↔ identifiant de
code (anglais). Un terme entre au glossaire quand : deux échanges l'ont
utilisé différemment, un pas de UC dépend de lui, ou c'est un faux ami.
L'agent relève les candidats ; l'utilisateur arbitre et rédige.

| Terme (FR) | Identifiant (code) | Définition (une phrase) |
|---|---|---|
| hibernation | `hibernate` | Mise hors tension prolongée : la RAM est écrite sur le disque, la machine s'éteint — à distinguer de la *suspension* (en RAM, consommation résiduelle). |
| seuil | `threshold` | niveau de batterie choisi par l'utilisateur comme niveau minimum de batterie avant mise en hibernation |
| barre d'état système | `tray` | zone d'affichage des icônes système (ex. sur Linux Mint : en bas à droite à côté de l'heure) |
| fenêtre de configuration | `config_window` | fenêtre de l'application dans laquelle l'utilisateur peut changer la configuration de l'application (seuil de batterie par exemple) |
| paramètre | `parameter` | valeur réglable de l'application (ex. le seuil de batterie), affichée dans la fenêtre de configuration et destinée à persister entre les lancements |
| Enregistrer | `save` | bouton de la fenêtre de configuration qui persiste les paramètres modifiés puis referme la fenêtre |
| Annuler | `cancel` | bouton de la fenêtre de configuration qui referme la fenêtre en abandonnant les modifications non enregistrées |