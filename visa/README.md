#  Virtual instrument software architecture (VISA)
This is a wrapper around the native implementations of Visa from multiple vendors. This wrapper allows for concurrent use of different visa implemntations or dynamic switching between them during runtime if needed. This library is kept as close as possible to native implementation so the user will need to use CTypes such as CString, and[u8;x] arrays, c_char, c_uchar, c_schar, c_void etc. This library can be used as is or if you prefer a safe simplified abstraction then you can use instrument_communication library which will be published later this year 2023. 

# How to use
To use this library you can load a visa dynamically linked library .dll or .so etc. Keysight option targets the binary name that is installed by default on the target platform when using the official keysight installer. Similarly NiVisa is for National Instrument visa. The "Primary" option targets whichever implementation currently serves as the default visa implementation. On windows 64 the primary is C:\Windows\System32\visa32.dll.
The method below sequantially tries to load each in order and returns on first success or last failure.
```rust
 let visa = visa::create(&visa::Binary::Keysight)
 .or_else(|_| visa::create(&visa::Binary::NiVisa))
 .or_else(|_| visa::create(&visa::Binary::Primary))
 .or_else(|_| visa::create(&visa::Binary::Custom("visa.so".into())));
```
then you need to open a default session
```rust
let mut _session = 0;
let status = visa.viOpenDefaultRM(&mut _session);
```

once that's open, you can try connecting to an instrument using its address
```rust
let address = CString::new(format!("TCPIP0::{IPADDRESS}::{PORT}::SOCKET"))?;
let mut vi = 0;
let status = visa.viOpen(_session, address.as_ptr(), 0, 0, &mut vi);
```

note a successfully connection will return a status of 0. You can then set the timeout and termination charachter
```rust
visa.viSetAttribute(vi, visa::VI_ATTR_TMO_VALUE, 5000); // Set timeout
visa.viSetAttribute(vi, visa::VI_ATTR_TERMCHAR, 10); // set termination byte to 10
visa.viSetAttribute(vi, visa::VI_ATTR_TERMCHAR_EN, 1); // enabled termination byte to stop reading when encountering this character.
```

define the command string as byte array.
```rust
let cmd = b"*IDN?\n";
```
initialize return character count
```rust
let mut ret_cnt = 0u32;
```
write command to instrument
```rust
let status=visa.viWrite(vi,cmd.as_ptr(),u32::try_from(cmd.len())?,&mut ret_cnt);
```
status will be 0 if successfull. 

Define read buffer size (50 bytes in this case)
```rust
let resp = vec![0u8; 50];
```
Then read the return message
```rust
let status = visa.viRead(vi, resp.as_ptr() as *mut _, 50, &mut ret_cnt);
```
convert the bytes to a readable text.
```rust
let response = std::str::from_utf8(&resp[0..ret_cnt as usize])?;
```
print it 
```rust
println!("Response : {}", response);
```

