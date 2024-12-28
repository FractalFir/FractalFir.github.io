Some toughts on reflection in Rust

Ah, reflection.

There is some sort of strange, exotic beauty in this odd programing paradigm. I can't explain why, but it has captured my attention for quite some time now.
I find myself obsessed with this missing feature of Rust, drafting and scrapping design after design, idea after idea.

At first glance, it may seem like I don't have much to show for my strange obseesion. I feel like none of my designs were nearly good enough. I am a bit of a perfectionist, and
tend to find and get stuck on all sorts of flaws and ege cases. 

Still, I feel like I can still share some things I discovered along the way. While quite a few of the issues I encountered were specific to my designs, other issues tend to be more general,
some of them even being inherent to the Rust language. 

# Respecting privacy

In quite a few languages, reflection has additional superpowers. It not only enables you to inspect the type system or generate code,
it also allows you to violate some of the rules of the language you are using. 

Quite often, reflection can access private fields of types. 
On one hand, this can cause quite a few issues. After all, those fields are private for a reason, and we would not like for anyone to mess with them.

## Violating privacy is not good

Violating access rules allows us to do some truly cursed things. 

Have you heard of the `sun.misc.Unsafe`? This Java class allows you to do a lot of cool things. Have you ever wanted to use raw pointers and manual memory managament in Java?
Well, with `Unsafe` you can... sort of. You can't *normally* get any instances of this class, which prevents you from using it. 

The only instance of this class is inconvinently hidden in a **private** static field.

Curses! Is your dream of pointers in Java dead?

Nope! We can just do this:
```java
// Get the field descriptor trough reflection
Field fd = Unsafe.class.getDeclaredField("theUnsafe");
// Private? I don't think so!
fd.setAccessible(true);
// Just get the value of the static field
Unsafe unsafe = (Unsafe) fd.get(null);
```
and be on our merry way, scaring peopele with our new found unsafe superpowers. 

As you can see, allowing peopele to access private class members can have some unexpected consequences. We break encapsulation, and can even violate certain invariants if we wish to
do so. 

## No privacy is good, akcsualy 

With all of this said, allowing reflection to violate traditional access rules is sometimes a good thing.

For example, if we want to serailze and deserialze this class:
```java
class Deser implements java.io.Serializable
{
    private int a;
    private String b;
}
```
we need some way to acces private fields. Equaly, if Rust had a reflection API, we would need some way to access fields if we wanted to implement serailzation using reflection.

Let us imagine we want to serialize a struct called `Even` form an external crate. 
```rust
struct Even(i32);
impl Even{
    pub fn new(val:i32)->Option<Self>{
        if val % 2 == 0{
            Some(Self(val))
        }
        else { 
            None
        }
    }
}
```

The internal value of this struct is *private* and it should stay so. We did all of that hard work ensuring this struct could only contain even numbers, and we don't want anybody to
mess with that.

Still, we want to be able to serialzie and deserialize this struct. If reflection can't access the fields of this struct, then we can't use it to serialze this data. 
Since this is something peopele want to use reflection for, we should at least consider it to some degree. After all, what good is a feature that nobody uses?

Surely, nothing bad can come from violating access rules a little bit?

# Violating privacy can be either safe or sound.

Sadly, I regret to inform you that Rust **really** does not like when access rules are not followed. 

Let's say we write, using reflection, a simple function like this:

```rust
/// For any given type `T`, returns a mutable reference to its 1st field of type `F`.
fn get_first_field_mut<T,F>(t:&mut T)-> &mut F;
```

At first glance, it does not look too bad. Sure, it is a little bit cursed, but there is nothing inherently unsafe or unsound about it. 
It respects all the borrowing rules, has proper lifetimes - all seems fine. This function *seems* like it should be safe. 

Without reflection, you could implement a function like that for all your types, using 100% safe Rust.
```rust
struct MyType(i32)
impl GetFirstFieldMutTrait{
    type F = i32;
    fn get_first_field_mut<T>(&mut self)-> &mut Self::F{
        &mut self.0
    }
}
```
However, despite its innocent apperance, being able to do this operation on arbitray types will quickly blow up in your face.

Let us start with the simpler `Even` example:
```rust
// Somebodys crate
struct Even(i32);
impl Even{
    pub fn new(val:i32)->Option<Self>{
        if val % 2 == 0{
            Some(Self(val))
        }
        else { 
            None
        }
    }
}
// Our code 
let even = Even::new(4).unwrap();
// Not so even anymore, eh?
*get_first_field_mut(even) = 17;
// Calls some C code trough ffi. This code assumes `even` is an even number. Should be safe, since `even` is... well, even.
blows_up_if_not_even(even);
```
This already spells toruble for accessing private fields torugh reflection 