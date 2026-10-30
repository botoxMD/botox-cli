# Exercice C — Algèbre relationnelle

## Schéma de la base de données

> **employee**($\underline{ssn}$, fname, minit, lname, bdate, address, sex, salary, super\_ssn, dno)
> — FK : super\_ssn → employee.ssn, dno → department.dnumber
>
> **department**($\underline{dnumber}$, dname, mgr\_ssn, mgr\_start\_date)
> — FK : mgr\_ssn → employee.ssn
>
> **dept\_locations**($\underline{dnumber, dlocation}$)
> — FK : dnumber → department.dnumber
>
> **project**($\underline{pnumber}$, pname, plocation, dnum)
> — FK : plocation → location.lnumber, dnum → department.dnumber
>
> **works\_on**($\underline{essn, pno}$, hours)
> — FK : essn → employee.ssn, pno → project.pnumber
>
> **dependent**($\underline{essn, dependent\_name}$, sex, bdate, relationship)
> — FK : essn → employee.ssn
>
> **location**($\underline{lnumber}$, lname)

---

## 1. Récupérer le SSN des employés du département 5 ou qui supervisent directement un employé de ce département.

*On commence par sélectionner les employés du département 5*
$$emp\_dept5 \leftarrow \sigma_{dno=5}(employee)$$

*Leurs SSN directs*
$$ssn\_dept5 \leftarrow \pi_{ssn}(emp\_dept5)$$

*Les SSN de leurs superviseurs (super_ssn des employés du dpt 5)*
$$super\_ssn\_dept5 \leftarrow \pi_{super\_ssn}(emp\_dept5)$$

*On renomme super_ssn en ssn pour que les schémas soient compatibles pour l'union*
$$superviseurs \leftarrow \rho_{(ssn)}(super\_ssn\_dept5)$$

*Union des deux ensembles*
$$\boxed{R \leftarrow ssn\_dept5 \cup superviseurs}$$

---

## 2. Trouver la liste des numéros des employés qui ne travaillent sur aucun projet.

*Tous les SSN d'employés*
$$tous\_ssn \leftarrow \pi_{ssn}(employee)$$

*Les SSN des employés qui travaillent sur au moins un projet (renommé essn → ssn)*
$$ssn\_avec\_projet \leftarrow \rho_{(ssn)}(\pi_{essn}(works\_on))$$

*Différence : tous les employés moins ceux qui travaillent sur un projet*
$$\boxed{R \leftarrow tous\_ssn - ssn\_avec\_projet}$$

---

## 3. Trouver toutes les paires (e1, e2) où l'employé plus jeune (e1) a un salaire supérieur à l'employé plus âgé (e2).

*On renomme une copie de employee pour le produit cartésien*
$$e2 \leftarrow \rho_{(ssn_2, fname_2, minit_2, lname_2, bdate_2, address_2, sex_2, salary_2, super\_ssn_2, dno_2)}(employee)$$

*Produit cartésien de toutes les paires d'employés*
$$paires \leftarrow employee \times e2$$

*e1 est plus jeune ⟹ bdate > bdate_2 (une date de naissance plus récente = plus jeune)*
*e1 a un salaire supérieur ⟹ salary > salary_2*
$$\boxed{R \leftarrow \sigma_{bdate > bdate_2 \;\wedge\; salary > salary_2}(paires)}$$

---

## 4. Extraire le nom des employés qui travaillent sur tous les projets sur lesquels travaille "John Smith".

*On trouve d'abord le SSN de John Smith*
$$john \leftarrow \sigma_{fname='John' \;\wedge\; lname='Smith'}(employee)$$
$$ssn\_john \leftarrow \pi_{ssn}(john)$$

*Les projets (pno) sur lesquels travaille John Smith (via produit cartésien + sélection)*
$$pno\_john \leftarrow \pi_{pno}(\sigma_{essn = ssn}(works\_on \times ssn\_john))$$

*Pour chaque employé, la liste de ses projets (essn, pno)*
$$emp\_projets \leftarrow \pi_{essn, pno}(works\_on)$$

### Solution 1 : avec l'opérateur de division

*Division : on ne garde que les essn qui sont associés à TOUS les projets de John Smith*
$$essn\_result \leftarrow emp\_projets / pno\_john$$

*On récupère le nom de l'employé via produit cartésien et sélection*
$$\boxed{R \leftarrow \pi_{fname, lname}(\sigma_{essn = ssn}(essn\_result \times employee))}$$

### Solution 2 : sans l'opérateur de division

*Définition de la division : $A / B = \pi_X(A) - \pi_X((\pi_X(A) \times B) - A)$*
*Ici : $A = emp\_projets(essn, pno)$, $B = pno\_john(pno)$, et $X = \{essn\}$*

*Tous les employés ayant au moins un projet*
$$tous\_essn \leftarrow \pi_{essn}(emp\_projets)$$

*Toutes les combinaisons possibles théoriques (chaque employé avec chaque projet de John)*
$$combinaisons \leftarrow tous\_essn \times pno\_john$$

*Combinaisons qui n'existent PAS dans works_on (les projets manquants pour chaque employé)*
$$manquants \leftarrow combinaisons - emp\_projets$$

*Employés à qui il manque au moins un projet*
$$essn\_incomplets \leftarrow \pi_{essn}(manquants)$$

*Employés qui ont tous les projets de John Smith*
$$essn\_complets \leftarrow tous\_essn - essn\_incomplets$$

*Projection finale du nom*
$$\boxed{R \leftarrow \pi_{fname, lname}(\sigma_{essn = ssn}(essn\_complets \times employee))}$$

---

## 5. Extraire le SSN, le nom et l'adresse de tous les employés qui travaillent pour le département "Recherche".

*On sélectionne le département "Recherche"*
$$dept\_rech \leftarrow \sigma_{dname='Recherche'}(department)$$

*Combinaison des employés avec le département Recherche (produit cartésien + sélection sur dno = dnumber)*
$$emp\_rech \leftarrow \sigma_{dno = dnumber}(employee \times dept\_rech)$$

*Projection des informations demandées*
$$\boxed{R \leftarrow \pi_{ssn, fname, lname, address}(emp\_rech)}$$

---

## 6. Pour chaque projet situé à "Stafford", indiquer le numéro du projet, le numéro du département qui en a la charge, le nom de famille, l'adresse et la date de naissance du directeur de département.

*1. Trouver l'identifiant de la localisation 'Stafford'*
$$loc\_stafford \leftarrow \sigma_{lname='Stafford'}(location)$$

*2. Projets situés à Stafford (liaison project et location sur plocation = lnumber)*
$$proj\_stafford \leftarrow \sigma_{plocation = lnumber}(project \times loc\_stafford)$$

*3. Département en charge de chaque projet (liaison sur dnum = dnumber)*
$$proj\_dept \leftarrow \sigma_{dnum = dnumber}(proj\_stafford \times department)$$

*4. Directeur (manager) de ce département (liaison sur mgr_ssn = ssn)*
$$proj\_mgr \leftarrow \sigma_{mgr\_ssn = ssn}(proj\_dept \times employee)$$

*5. Projection des colonnes demandées*
$$\boxed{R \leftarrow \pi_{pnumber, dnum, lname, address, bdate}(proj\_mgr)}$$

---

## 7. Trouver le nom et le SSN des employés qui travaillent sur tous les projets contrôlés par le département 5.

*Projets contrôlés par le département 5*
$$proj\_dept5 \leftarrow \pi_{pnumber}(\sigma_{dnum=5}(project))$$

*Renommer pnumber en pno pour correspondre à l'attribut de works_on*
$$pno\_dept5 \leftarrow \rho_{(pno)}(proj\_dept5)$$

*Associations (essn, pno) de works_on*
$$emp\_projets \leftarrow \pi_{essn, pno}(works\_on)$$

*Division : employés travaillant sur TOUS les projets du département 5*
$$essn\_result \leftarrow emp\_projets / pno\_dept5$$

*Récupérer le nom et SSN avec employee*
$$\boxed{R \leftarrow \pi_{fname, lname, ssn}(\sigma_{essn = ssn}(essn\_result \times employee))}$$

---

## 8. Liste des numéros des projets impliquant un employé nommé "Smith", soit comme travailleur soit comme directeur du département en charge.

*Identifier les employés nommés 'Smith'*
$$smith \leftarrow \sigma_{lname='Smith'}(employee)$$
$$ssn\_smith \leftarrow \pi_{ssn}(smith)$$

*Cas 1 : Projets sur lesquels un 'Smith' travaille*
$$pno\_travail \leftarrow \pi_{pno}(\sigma_{essn = ssn}(works\_on \times ssn\_smith))$$

*Cas 2 : Projets dont le département est dirigé par un 'Smith'*
*Départements dirigés par un Smith*
$$dept\_smith \leftarrow \pi_{dnumber}(\sigma_{mgr\_ssn = ssn}(department \times ssn\_smith))$$

*Projets associés à ces départements*
$$proj\_dept\_smith \leftarrow \pi_{pnumber}(\sigma_{dnum = dnumber}(project \times dept\_smith))$$

*Renommer pnumber en pno pour aligner le schéma avant l'union*
$$pno\_mgr \leftarrow \rho_{(pno)}(proj\_dept\_smith)$$

*Union des deux cas*
$$\boxed{R \leftarrow pno\_travail \cup pno\_mgr}$$

---

## 9. Récupérer le nom et le SSN des employés qui n'ont aucun ayant droit.

*Tous les SSN d'employés*
$$tous\_ssn \leftarrow \pi_{ssn}(employee)$$

*Les SSN ayant au moins un ayant droit (renommé en ssn pour compatibilité)*
$$ssn\_avec\_dep \leftarrow \rho_{(ssn)}(\pi_{essn}(dependent))$$

*Différence : ceux sans ayant droit*
$$ssn\_sans\_dep \leftarrow tous\_ssn - ssn\_avec\_dep$$

*Récupérer le nom et prénom en reliant avec employee*
$$\boxed{R \leftarrow \pi_{ssn, fname, lname}(\sigma_{ssn = ssn\_sans\_dep.ssn}(employee \times ssn\_sans\_dep))}$$

---

## 10. Trouver le nom et le SSN des directeurs de département qui ont au moins un ayant droit.

*Les SSN des directeurs de département (renommés en ssn)*
$$mgr\_ssn\_set \leftarrow \rho_{(ssn)}(\pi_{mgr\_ssn}(department))$$

*Les SSN des employés ayant au moins un ayant droit (renommés en ssn)*
$$ssn\_avec\_dep \leftarrow \rho_{(ssn)}(\pi_{essn}(dependent))$$

*Intersection : directeurs qui ont au moins un ayant droit*
$$mgr\_avec\_dep \leftarrow mgr\_ssn\_set \cap ssn\_avec\_dep$$

*Récupérer le nom et prénom via employee*
$$\boxed{R \leftarrow \pi_{ssn, fname, lname}(\sigma_{ssn = mgr\_avec\_dep.ssn}(employee \times mgr\_avec\_dep))}$$

---

## 11. Trouver les employés (SSN) qui ont le salaire le plus élevé.

*Idée : Trouver les employés dont le salaire n'est dépassé par aucun autre.*
*On élimine tous les employés dont le salaire est strictement inférieur à celui d'un autre.*

*Copie renommée d'employee*
$$e2 \leftarrow \rho_{(ssn_2, fname_2, minit_2, lname_2, bdate_2, address_2, sex_2, salary_2, super\_ssn_2, dno_2)}(employee)$$

*Paires où un employé gagne strictement moins qu'un autre*
$$paires \leftarrow employee \times e2$$
$$moins\_bien\_payes \leftarrow \pi_{ssn}(\sigma_{salary < salary_2}(paires))$$

*Résultat : ensemble de tous les employés moins ceux qui sont dépassés*
$$\boxed{R \leftarrow \pi_{ssn}(employee) - moins\_bien\_payes}$$
