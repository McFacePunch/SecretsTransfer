
# <table>
  <tr>
    <td><img src="./images/logo.png" alt="Project Logo" width="200" height="200"></td>
    <td><h1>Secrets Transfer</h1></td>
  </tr>
</table>
 

A secure secrets sharing service written in Rust and packaged neatly into a single binary for easy deployment and use. It provides an easy and minimal setup way to share secrets in a reliable and secure way.


While being capable of running standalone with no extra infrastructure it can also be deployed as part of a cluster with caching using Redis/Valkey.

Key Features:
- Easy to setup and run, only needs one binary and a config
- Pure Rust internals, no unsafe code. 
- Minimal Javascript
- API for building custom integrations
- Secure transfer of secrets
- All features exposed via CLI (In Progress)
- Optional Captcha (TODO)
- HTML templatization for easy customization
- Open Source and free to use.



