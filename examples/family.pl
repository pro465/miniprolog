% taken from here: https://en.wikipedia.org/wiki/Prolog#Execution
mother_child(trude, sally) :- true.
 
father_child(tom, sally) :- true.
father_child(tom, erica) :- true.
father_child(mike, tom) :- true.
 
sibling(X, Y)      :- parent_child(Z, X), parent_child(Z, Y).
 
parent_child(X, Y) :- father_child(X, Y).
parent_child(X, Y) :- mother_child(X, Y).
