pub fn print(version: &str) {
    let b = r"                  
      ___           ___           ___           ___           ___         
     /\  \         /\  \         /\  \         |\__\         |\__\        
    /::\  \       /::\  \       /::\  \        |:|  |        |:|  |       
   /:/\:\  \     /:/\:\  \     /:/\:\  \       |:|  |        |:|  |       
  /::\~\:\  \   /::\~\:\  \   /:/  \:\  \      |:|__|__      |:|__|__     
 /:/\:\ \:\__\ /:/\:\ \:\__\ /:/__/ \:\__\ ____/::::\__\     /::::\__\    
 \/__\:\/:/  / \/_|::\/:/  / \:\  \ /:/  / \::::/~~/~       /:/~~/~       
      \::/  /     |:|::/  /   \:\  /:/  /   ~~|:|~~|       /:/  /         
       \/__/      |:|\/__/     \:\/:/  /      |:|  |       \/__/          
                  |:|  |        \::/  /       |:|  |                      
                   \|__|         \/__/         \|__|  
                        
                   ";
    println!("{}                {}", b, version);
}