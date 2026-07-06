# Politique de sécurité

## Versions supportées

Seule la branche `main` reçoit des correctifs de sécurité tant
qu'aucune release stable (`v1.0.0`) n'est publiée.

## Signaler une vulnérabilité

Merci de **ne pas** ouvrir d'issue publique pour une vulnérabilité.

Utilise la fonction **"Report a vulnerability"** de l'onglet
Security du repo (private security advisory), ou contacte les
mainteneurs directement.

On répond sous 7 jours ouvrés. Une fois le fix disponible, un
avis public est publié avec crédit au rapporteur (sauf demande
contraire).

## Surface d'attaque

Adventure Timing est un jeu solo qui :
- lit/écrit du JSON dans le user data dir (`save.rs`),
- charge des assets locaux depuis `./assets/`.

Il n'y a pas de réseau, pas de multijoueur, pas de code
utilisateur exécuté. Le vecteur principal est un fichier `save.json`
ou un asset malicieux fourni au binaire.
