initSidebarItems({"mod":[["conn","Contains various type and trait definition related to connexion management"],["defs","Various definitions of types, constants, traint, ... related to network"]],"struct":[["Builder","Wrap packet creationThis object is convenient to create a new packet. It wraps allocation, buffer management and final format of the packet. The packet yielded by the `finalize` method can be directly sent out on the network."],["Instance","A network stackThis object represent a shareable network stack. This object stack internally uses an Arc so it can be safely shared."],["InstanceWeak","A weak reference over a network stack"],["Interface","A shareable network interface.This is backed by an Arc and a RwLock so that accesses are safe and the entity can be shared."],["InterfaceRaw","The raw network interface"],["InterfaceWeak","A interface weak reference"],["MultiConn","Connexion that can receive packets from multiple endpoints.This is just a placeholder object. In order for it to receive packets it must be added to a `Manager`."],["Packet","A network packet"],["Stack","Uni.rs network stack"],["UniConn","Connexion the can receive packet from a single endpoint"],["V4Configuration","IPv4 configuration of an interface"]],"trait":[["Formatter","Used to format a packet at the link or network layer"]]});