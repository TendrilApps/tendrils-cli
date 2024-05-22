case "$OSTYPE" in
  solaris*) echo "solaris" ;;
  darwin*)  echo "osx" ;; 
  linux*)   echo "linux" ;;
  bsd*)     echo "bsd" ;;
  msys*)    echo "windows" ;;
  cygwin*)  echo "windows" ;;
  *)        echo "unknown: $OSTYPE" ;;
esac
