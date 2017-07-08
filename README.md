#  ViewDB

#### ViewDB is a *fact ⇒ domain* inference database.

It means that the only *source of truth* data recorded in ViewDB is 
**facts**: observed events that happened at some point in time. It
indexes the **attributes** of these facts so that domain models can
be effectively represented through queries.

This allows to forgo careful domain schema planning and focus on
accurate recording of facts, knowing that these models can be easily
changed and adapted to new requirements. What is equally important,
this allows larger systems to have overlapping, and sometimes even
mutually exclusive domain models at the same time, as they are effectively 
just "views" on facts.

At its heart, this approach is not too dissimilar from the concept of 
*event sourcing*. However, it prioritizes late and lazy domain binding:
the ability to produce the domain models on-demand without replaying all
events, or having to designate individual event streams upfront.

> Here's an example. Let's assume that a person who uses the application,
> decided to change the name in their profile. This will ultimately result
> in a `NameChanged` fact with an attribute for the new name, an attribute
> for the reference to the person's unique identifier and an attribute
> for the timestamp of the effective change.

> Now, for the purpose of rendering this person's name in their profile,
> we need to update the query to find the latest `NameChanged` fact that refers
> to this person.

> At the same time, the domain model for the fraud prevention component
> is different, as it isn't particularly interested in the up-to-date person's
> name. It is rather interested in the frequency of name changes and past names
> themselves to cross-reference them with other sources. So, the query that
> defines that domain model will reflect these needs, independently from the model
> used the previous case.

## Architecture

At the core, ViewDB is very simple. It's built on top of
[PumpkinDB](http://pumpkindb.org) and focuses entirely on the data modelling
aspects, delegating all the lower-level responsibilities to PumpkinDB.

### Fact

Uniquely identifiable observed event. By itself, does not contain any data
besides the identifier.

### Attribute

This is what *attaches* a single piece of data to a fact. Attribute has a
binary identifier. Most commonly this identifier would be a UTF-8 string,
often a URL, but there could be more complicated cases. 

For example, an `https://viewdb.org/attributes#timestamp` attribute can
be used to add a timestamp to a fact. 

Another common case is declaring a *fact type* convention through an
`https://viewdb.org/attributes#factType` attribute. This greatly simplifies
querying in a majority of cases.

Attribute values that are attached to a particular fact are (again) just
simple binaries. However, an attribute definition can optionally put more
restrictive constraints onto the values. We'll get to this later.

An interesting property of attribute value attachments is that they can be
done over the course of multiple transactions, separated in time. This means
that in cases where the entirety of information is not available at the original
transaction's time, more information can be attached to that fact at a later
time.

Another important aspect of ViewDB's implementation is that for every attribute
value attachment, it associates current transaction identifier with it. In ViewDB,
transaction IDs are unique and grow monotonically (it's a property ViewDB
inherits from PumpkinDB). This allow us to tell if two attributes were attached
in the same transaction or, if they were not, which one precedes the other.

### Trait

Trait is a set of attributes with some values (optionally) constant. A good
example of this would be building a trait over a fact type. We can define our 
`NameChanged` trait over a `NameChanged` fact type this way (pseudo-code):

```
trait NameChanged {
   "https://viewdb.org/attributes#factType" => "NameChanged",
   "https://viewdb.org/attributes#value",
}
```

In the above example, we have only specified the constant value for the fact
type, but not for the rest of the attributes. Did you notice, however, that we
still don't have a reference to *whose* name was changed? This is because the above
trait was only concerned with the fact of the name change itself, but nothing else. 

This is where the power of traits comes in. We can define additional traits that
help us understanding the information better. In this case, instead of adding yet
another attribute to the `NameChanged` trait, we can define a generic `Object` trait 
(and re-use in all other cases):

```
trait Object {
    "https://view.org/attributes#object",
}
```

This extensibility can go further. In a very similar way we can, for example, 
designate a `Subject` trait that would allow us to describe *who* or *what* 
effected the change.

Traits are essentially a thin "pattern matching" layer that makes comprehending
factual information easier for domain experts, software architects and
developers.

### Domain Identifiers

You might have noticed that we were mentioning "references" to a person yet
didn't say how do they look like? Are they UUIDs? Serial keys? 

This omission was kind of intentional. The idea here is to leave this a bit
more open to application designers, as there are different situation and one
solution won't fit all.

For example, for a person, a relatively "loose" reference to it can be an email
address. Sure, it won't work if the person changes the address — assuming there's
an account, but if there's no account, the email might serve just fine. 

Another more universal approach is to use a "first introduction fact" convention
(FIF). That is to say, whichever fact first introduced the object of a reference,
its unique identifier can be used to identify the entity (or multiple entities in
different domains, if applicable) of that object. So, for example, a fact with a 
`AccountCreated` can be used as a source of the identifier. 

An even subtler (but not less important) identification mechanism is called 
"lazy introduction by referencing entity" (LIBRE). Once the system becomes "aware"
of a certain entity through the first fact that references the entity, a unique
identifier can be generated and used for that first and following references.
To illustrate this, we can take the `NameChanged` example again. Suppose we know
nothing about the person using our application (anonymous user) and this is the
first time the person decided to provide their name. We can now generate a new unique
identifier, and reference it in the value of the `https://viewdb.org/attributes#object`
attribute. Now, once that same person uses a similar mechanism to provide their email
address, we can now use this information to connect the internal unique identifier to
the email. For example, when the person supplies their email address during a sign-in,
we can find the last fact with `EmailChanged` and `Object` traits and, subsequently,
retrieve more information about this person through the identifier available in the
`Object` trait.

### Querying

So, how do we query the facts to build our models? It's actually pretty simple. A query
is essentially a list of traits with testing conditions on attribute values and variable
bindings. You can imagine doing something like this to retrieve person's up-to-date name
(again, pseudo-code):

```
SELECT ?Name, MAX(?Timestamp) WHERE
       Object("https://view.org/attributes#object" = PersonId) AND 
       NameChanged("https://viewdb.org/attributes#value" = ?Name) AND
       Timestamp("https://viewdb.org/attributes#timestamp" = ?Timestamp) 
```

In the above example, we bind `?Name` and `?Timestamp` to queried values and return them
as a result.

## Status

ViewDB source code has not been published yet, but this is expected to happen during the
summer of 2017.