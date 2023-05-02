import subprocess, hashlib, base64
from getpass import getpass

challenge = getpass(prompt="Challenge: ")

bashCommand = f"ykchalresp -2 {challenge}"
process = subprocess.Popen(bashCommand.split(), stdout=subprocess.PIPE)
output, error = process.communicate()

bresp = base64.b16decode(output.decode("utf8").strip().upper())

passphrase = getpass(prompt="Passphrase: ")
bpassphrase = passphrase.encode("utf-8")
hpassphrase = hashlib.sha1(bpassphrase)

q = bytes(a ^ b for (a, b) in zip(bresp, hpassphrase.digest()))

with open("secret", "wb") as secret:
    secret.write(q)
