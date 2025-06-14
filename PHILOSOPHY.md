# 1. Screaming architecture

Refuse names like controllers, models, views etc. The file names and structures should scream about their intent. If this is a tool about user management, you should have names like user, accesses, roles etc. 

# 2. Low abstractions

An abstraction is a trait, interface, type, function and anything that hides logic or intention behind a name. There are only two good reasons for abstractions: domain representation (ie. structure or use-case) and DRY principle.

# 3. Sort things alphabetically

There are many ways to order things. Perhaps dates at the end, identifiers at the top, some tag somewhere around the top. But when the thing evolves, we end up pushing new things at the end. The initial well-though organization is lost and no-one understand it anymore. 

The only arbitrary way to organize thing unequivocally is alphabetical order. Is it the best order? No. Is it well understood by everyone and almost applies in almost 100% of situation without even having to think about it? Yes.
