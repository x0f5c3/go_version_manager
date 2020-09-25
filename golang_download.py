import git
from bs4 import BeautifulSoup
import requests
from distutils.version import LooseVersion
import re
from packaging import version
import hashlib
import platform
import pathlib
from tqdm import tqdm
import elevate
import tarfile
import subprocess
def get_arch():
    raw_arch = platform.machine()
    if raw_arch == "x86_64":
        return "amd64"
    else:
        return raw_arch
arch = get_arch()
os = platform.system().lower()
g = git.cmd.Git()

def get_latest_version():
    tags = []
    pattern = re.compile("^go")
    for ref in g.ls_remote("--tags","https://github.com/golang/go").split('\n'):
        hash_ref_list = ref.split('\t')
        splitted = hash_ref_list[1].split('/')[2]
        if pattern.match(splitted):
            tags.append(splitted.replace("go",""))
    sortd = sorted(tags,key=lambda x: version.Version(x),reverse=True)
    latest = "go" + sortd[0]
    return latest



def get_path_and_hash(version):
    resp = requests.get("https://golang.org/dl/")
    bs = BeautifulSoup(resp.text,"lxml")
    divs = bs.find(id=version)
    trs = divs.find_all('tr')
    files_hashes = {}
    for tag in trs:
        file_tag = tag.find('td', class_="filename")
        if file_tag:
            file_maybe = file_tag.next_element.get('href')
            if file_maybe:
                file = file_maybe.split('/')[2]
        else:
            file = "None"
        sha_tag = tag.find('tt')
        if sha_tag:
            sha = sha_tag.string
        else:
            sha = "None"
        files_hashes[file] = sha
    return files_hashes
def get_download(versions,system,arch):
    regex = re.compile(".*" + system + "-" + arch)
    for file in versions:
        if regex.match(file):
            url = "https://golang.org/dl/" + file
            to_check = versions[file]
            currentpath = str(pathlib.Path().absolute())
            path = currentpath + "/" + file
            return (path,to_check, url)  

def download(url, path):
    with open(path, 'wb') as f:
        resp = requests.get(url, stream=True,allow_redirects=True)
        total = resp.headers.get('content-length',0)
        file_name = path.split('/')[5]
        print(file_name)
        if total is None:
            f.write(resp.content)
        else:
            with tqdm(unit='B', unit_scale=True, unit_divisor=1024, miniters=1,
            desc=file_name, total=int(total)) as pbar:
                for data in resp.iter_content(chunk_size=4096):
                    f.write(data)
                    pbar.update(len(data))
        
def check_hash(path, sha):
        h = hashlib.sha256()
        with open(path,'rb') as f:
            fb = f.read(65536)
            while len(fb) > 0:
                h.update(fb)
                fb = f.read(65536)
        if sha == h.hexdigest():
            print("Nice")
def install(path,os):
    if os == "linux":
        tar = tarfile.open(path)
        elevate.elevate(show_console=False,graphical=False)
        tar.extractall("/usr/local/")
    if os == "windows":
        subprocess.call([path, '-ARG'], shell=True)



latest = get_latest_version()
hashes = get_path_and_hash(latest)
(path, sha, url) = get_download(hashes,os,arch)
download(url,path)
check_hash(path,sha)


