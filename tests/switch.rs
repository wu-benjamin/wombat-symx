fn test(x_mut: i32) -> i32 {
    if x_mut > 0 {
        println!("Positive!")   
    }
    match x_mut{
        1=>{println!("One");
            x_mut
        },  
        2=>{println!("Two");
            x_mut
        },
        3=>{println!("Three");
            x_mut-1
        },
        _=>x_mut+1
    }
}


fn main() {
    println!("Hello, world!");
    test(5);
}
