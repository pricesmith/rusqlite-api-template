# TLM Server



### quirk and dirty packet receiving server.  

Accepts a multipart file upload, wants "metadata" and "packet" files.  


Here's the test with curl:


file metadata.json has some data.  there is also a packet.bin file with some binary data ("are we the baddies" jpeg)

```
bburdette@BB-5520:~/code/outpost.space/rust-server/server/uptest$ cat metadata.json
{"name": "Tom", "age": 35, "favoriteSport" : "Foosball"}
```

'uptest.sh'  shell script uploads metadata.json and packet.bin with names 'metadata' and 'packet'

```
bburdette@BB-5520:~/code/outpost.space/rust-server/server/uptest$ cat uptest.sh
curl -F "metadata=@metadata.json" -F "packet=@packet.bin"  http://localhost:8000/upload
```

running uptest.sh returns a 'null of success'.

```
bburdette@BB-5520:~/code/outpost.space/rust-server/server/uptest$ ./uptest.sh
null
```

run sqlite on the database file, select from table level0blobs, our row is in there.  
  
```
bburdette@BB-5520:~/code/outpost.space/rust-server/server/uptest$ sqlite3 ../outpost.db
SQLite version 3.37.2 2022-01-06 13:25:41
Enter ".help" for usage hints.
sqlite> select * from level0blobs;
1|47e64bac-f865-4681-bc69-19a40a19385a|1670815311000|{"name": "Jack", "age": 30, "favoriteSport" : "Football"}
|��
   2|854a932b-3168-4291-bb05-3739ce4e9f66|1670815702000|{"name": "Tom", "age": 35, "favoriteSport" : "Foosball"}
|��
sqlite> 
```
# rusqlite-api-template
