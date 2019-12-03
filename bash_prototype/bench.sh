#!/bin/bash

function set_poolName(){
	skipNext=false
	for p in $@
	do	
		if $skipNext 
		then
			#echo skipping $p
			skipNext=false
			continue
		fi
		#echo p is $p
		offset=1
		regex='[-].*'
		if [[ $p =~ $regex ]]
		then
			#echo regexmatch $p
			skipNext=true
		else
			poolName=$p
			break;
		fi
	done
	#echo $poolName
}

function do_test ()
{
	devSource=/dev/urandom
	set_poolName $@
	
	if [[ -r /$poolName ]] 
	 then sudo zpool destroy $poolName 
	fi ;
	 echo zpool create $@ ; 
	 sudo zpool create $@ ; 
	 if [[ "$?" != "0" ]] ; then exit 1 ; fi
	 sudo zpool status $poolName
	 sudo chmod 777 /$poolName/ 
	 if [[ "$?" != "0" ]] ; then exit 1 ; fi
	 
	 
	 if [[ "$?" != "0" ]] ; then exit 1 ; fi
		#totalBytes=4294967296 #4GiB
		totalBytes=536870912 #512Meg
		#totalBytes=67108864 #64Meg
		#bss=( "512" "1024" "2048" "4096" "8192" "1048576" "2097152" "4194304" "16777216" "67108864" )
        bss=( "512" "1024" "16384" "1048576" "16777216" "67108864" )
        echo "blocksize,	write[MB/s],	read[MB/s]"
		for bs in ${bss[@]}
		do
			count=`echo "scale=0; $totalBytes/$bs"|bc`
			#echo using bs=$bs count=$count
#            dd if=$devSource of=/$poolName/test.iso bs=$bs count=$count  conv=sync 2>&1 
            writeSpeed=`dd if=$devSource of=/$poolName/test.iso bs=$bs count=$count  conv=sync 2>&1 | grep -v records | grep -v warning | cut -d , -f 4 | cut -d ' ' -f 2`
			#kill the cache
			sudo zpool export $poolName
			sudo zpool import $poolName
#            dd if=/$poolName/test.iso of=/dev/null bs=$bs count=$count 2>&1 
            readSpeed=`dd if=/$poolName/test.iso of=/dev/null bs=$bs count=$count 2>&1  | grep -v records | grep -v warning | cut -d , -f 4 | cut -d ' ' -f 2`
            echo "$bs,	$writeSpeed,	$readSpeed" | awk -F"," '{printf "%9s,%9s,%9s\r\n",$1,$2,$3}'
		done
		
		
	 if [[ "$?" != "0" ]] ; then exit 1 ; fi

	if [[ -r /$poolName ]] 
	 then 
	 #sudo zpool destroy $poolName
	 echo ""
	fi ;
}

function test_eight450(){
    do_test eight450 /dev/sd{j,m,n,o,p,q,r,s}
    do_test eight450 mirror /dev/sd{j,m,n,o} mirror /dev/sd{p,q,r,s}
    do_test eight450 mirror /dev/sd{j,m} mirror /dev/sd{n,o} mirror /dev/sd{p,q} mirror /dev/sd{r,s}
    do_test eight450 raidz1 /dev/sd{j,m,n,o,p,q,r,s}
    do_test eight450 raidz1 /dev/sd{j,m,n,o} raidz1 /dev/sd{p,q,r,s}
    do_test eight450 raidz2 /dev/sd{j,m,n,o,p,q,r,s}
    do_test eight450 raidz3 /dev/sd{j,m,n,o,p,q,r,s}

    do_test -o ashift=12 eight450 /dev/sd{j,m,n,o,p,q,r,s}
    do_test -o ashift=12 eight450 mirror /dev/sd{j,m,n,o} mirror /dev/sd{p,q,r,s}
    do_test -o ashift=12 eight450 mirror /dev/sd{j,m} mirror /dev/sd{n,o} mirror /dev/sd{p,q} mirror /dev/sd{r,s}
    do_test -o ashift=12 eight450 raidz1 /dev/sd{j,m,n,o,p,q,r,s}
    do_test -o ashift=12 eight450 raidz1 /dev/sd{j,m,n,o} raidz1 /dev/sd{p,q,r,s}
    do_test -o ashift=12 eight450 raidz2 /dev/sd{j,m,n,o,p,q,r,s}
    do_test -o ashift=12 eight450 raidz3 /dev/sd{j,m,n,o,p,q,r,s}

}



function test_six8tb(){
    do_test six8tb mirror /dev/sd{f,g} mirror /dev/sd{h,i} mirror /dev/sd{t,u}
    do_test six8tb raidz1 /dev/sdf /dev/sdg /dev/sdh raidz1 /dev/sdi /dev/sdt /dev/sdu
    do_test six8tb raidz2 /dev/sd{f,g,h,i,t,u}
    do_test six8tb raidz1 /dev/sd{f,g,h,i,t,u}
    do_test six8tb /dev/sd{f,g,h,i,t,u}

    do_test -o ashift=12 six8tb mirror /dev/sd{f,g} mirror /dev/sd{h,i} mirror /dev/sd{t,u}
    do_test -o ashift=12 six8tb raidz1 /dev/sdf /dev/sdg /dev/sdh raidz1 /dev/sdi /dev/sdt /dev/sdu
    do_test -o ashift=12 six8tb raidz2 /dev/sd{f,g,h,i,t,u}
    do_test -o ashift=12 six8tb raidz1 /dev/sd{f,g,h,i,t,u}
    do_test -o ashift=12 six8tb /dev/sd{f,g,h,i,t,u}

}

test_eight450
test_six8tb

